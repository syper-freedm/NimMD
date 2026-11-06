//! C ABI для Swift Quick Look: рендер HTML и безопасное чтение картинок.
//!
//! Паники ядра ловятся `catch_unwind` на потоке со стеком 64 МБ (спек 5.2, В4),
//! чтобы не ронять appex. Байты файла декодируются `from_utf8_lossy` (Р11).

#![warn(missing_docs)]

use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use nimmd_assets::{AssetBudget, read_asset};
use nimmd_core::text::decode_lossy;
use nimmd_core::{AssetMode, FileKind, MathMode, RenderOptions, Theme, render};

/// Версия раскладки [`NimmdRenderOptions`]: неизвестная → отказ (NULL).
pub const NIMMD_RENDER_OPTIONS_VERSION: u32 = 1;

/// Размер стека рабочего потока FFI (спек 5.2).
const FFI_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Владеющий буфер байт как `Box<[u8]>` (Р26): `data` + `len`, освобождать [`nimmd_buffer_free`].
#[repr(C)]
pub struct NimmdBuffer {
    /// Указатель на `len` байт; не NULL при `len > 0`.
    pub data: *mut u8,
    /// Число байт по `data`.
    pub len: usize,
}

/// Опции рендера; `version` — первое поле для расширяемости ABI.
#[repr(C)]
pub struct NimmdRenderOptions {
    /// Должно быть [`NIMMD_RENDER_OPTIONS_VERSION`].
    pub version: u32,
    /// `0` Auto, `1` Light, `2` Dark.
    pub theme: u8,
    /// `0` Katex, `1` MathML, `2` Source.
    pub math: u8,
    /// `0` None, `1` Link (`base_url`).
    pub assets: u8,
    /// `0` — удалённые картинки плейсхолдером, иначе разрешены.
    pub remote_images: u8,
    /// `0` — [`nimmd_core::text::MAX_BYTES`], иначе предел в байтах.
    pub max_bytes: u64,
    /// База для `assets == 1`; UTF-8 C-строка или NULL.
    pub base_url: *const c_char,
    /// Расширение без точки (`md`, `toml`, …) или NULL → Markdown.
    pub extension: *const c_char,
}

struct AssetState {
    doc: Option<PathBuf>,
    budget: AssetBudget,
}

impl AssetState {
    fn new() -> Self {
        Self {
            doc: None,
            budget: AssetBudget::new(),
        }
    }

    fn budget_for(&mut self, doc: &Path) -> &mut AssetBudget {
        if self.doc.as_ref().is_none_or(|d| d != doc) {
            self.doc = Some(doc.to_path_buf());
            self.budget = AssetBudget::new();
        }
        &mut self.budget
    }

    fn reset_for(&mut self, doc: &Path) {
        self.doc = Some(doc.to_path_buf());
        self.budget = AssetBudget::new();
    }
}

// ponytail: один бюджет на процесс — каждый FFI-вызов поднимает новый поток 64 МБ
fn asset_state() -> &'static Mutex<AssetState> {
    static STATE: OnceLock<Mutex<AssetState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(AssetState::new()))
}

/// Тестовый крючок: следующий вызов `nimmd_render_html` паникует внутри `catch_unwind`.
static PANIC_NEXT_RENDER: AtomicBool = AtomicBool::new(false);

/// Включает одноразовую панику в следующем `nimmd_render_html` (только тесты).
#[cfg(test)]
pub fn test_arm_panic_next_render() {
    PANIC_NEXT_RENDER.store(true, Ordering::SeqCst);
}

/// Рендерит байты файла в полный HTML-документ.
///
/// Декодирует вход через `String::from_utf8_lossy` и выставляет баннер `not_utf8` (Р11).
/// При панике, неизвестной `version` или сбое потока возвращает NULL.
///
/// # Safety
///
/// - `bytes` валиден для чтения `len` байт, либо `len == 0` (тогда `bytes` может быть NULL).
/// - `path` — NULL или валидная NUL-terminated UTF-8 C-строка (путь документа для бюджета картинок).
/// - `opts` — NULL (тогда значения по умолчанию) или валидный указатель на [`NimmdRenderOptions`];
///   строки внутри опций — NULL или валидные NUL-terminated UTF-8 C-строки.
/// - Результат освобождать только через [`nimmd_buffer_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_render_html(
    bytes: *const u8,
    len: usize,
    path: *const c_char,
    opts: *const NimmdRenderOptions,
) -> *mut NimmdBuffer {
    // SAFETY: caller гарантирует валидность `bytes`/`len` на время вызова.
    let input = unsafe { copy_bytes(bytes, len) };
    // SAFETY: `path` — NULL или валидная C-строка.
    let path_buf = unsafe { optional_path(path) };
    // SAFETY: `opts` — NULL или валидный `NimmdRenderOptions` со строками.
    let parsed = unsafe { parse_options(opts) };
    let Some(options) = parsed else {
        return ptr::null_mut();
    };

    match run_on_ffi_stack(move || render_html_inner(&input, path_buf.as_deref(), options)) {
        Ok(bytes) => buffer_from_slice(bytes),
        Err(()) => ptr::null_mut(),
    }
}

/// Читает локальную картинку относительно каталога документа (`nimmd-assets`).
///
/// При отказе или панике — NULL. Бюджет 20 МБ общий на один `doc_path` в процессе.
///
/// # Safety
///
/// - `doc_path` и `href` — валидные NUL-terminated UTF-8 C-строки (не NULL).
/// - Результат освобождать только через [`nimmd_buffer_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_read_asset(
    doc_path: *const c_char,
    href: *const c_char,
) -> *mut NimmdBuffer {
    // SAFETY: `doc_path` — валидная C-строка на время вызова.
    let doc = unsafe { c_path(doc_path) };
    let Some(doc) = doc else {
        return ptr::null_mut();
    };
    // SAFETY: `href` — валидная C-строка на время вызова.
    let href = unsafe { c_str(href) };
    let Some(href) = href else {
        return ptr::null_mut();
    };
    let href = href.to_owned();

    match run_on_ffi_stack(move || read_asset_inner(&doc, &href)) {
        Ok(Some(bytes)) => buffer_from_slice(bytes),
        _ => ptr::null_mut(),
    }
}

/// Освобождает буфер, полученный из [`nimmd_render_html`] / [`nimmd_read_asset`].
///
/// # Safety
///
/// - `buf` — NULL или указатель, ранее возвращённый этими функциями и ещё не освобождённый.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_buffer_free(buf: *mut NimmdBuffer) {
    if buf.is_null() {
        return;
    }
    // SAFETY: `buf` получен из `Box::into_raw` в `buffer_from_slice`.
    let owned = unsafe { Box::from_raw(buf) };
    if !owned.data.is_null() {
        // SAFETY: `data`/`len` описывают единственный `Box<[u8]>`, созданный вместе с буфером.
        let slice_ptr = ptr::slice_from_raw_parts_mut(owned.data, owned.len);
        // SAFETY: восстанавливаем `Box<[u8]>` для корректного drop.
        drop(unsafe { Box::from_raw(slice_ptr) });
    }
}

fn render_html_inner(input: &[u8], path: Option<&Path>, mut options: RenderOptions) -> Box<[u8]> {
    // Намеренная паника для проверки catch_unwind (в т.ч. release).
    assert!(
        !PANIC_NEXT_RENDER.swap(false, Ordering::SeqCst),
        "nimmd-ffi: test panic"
    );
    if let Some(p) = path {
        if let Ok(mut state) = asset_state().lock() {
            state.reset_for(p);
        }
    }
    let (text, not_utf8) = decode_lossy(input);
    options.not_utf8 = not_utf8;
    let out = render(&text, &options);
    out.html.into_bytes().into_boxed_slice()
}

fn read_asset_inner(doc: &Path, href: &str) -> Option<Box<[u8]>> {
    let mut state = asset_state().lock().ok()?;
    let budget = state.budget_for(doc);
    read_asset(doc, href, budget)
        .ok()
        .map(Vec::into_boxed_slice)
}

fn buffer_from_slice(bytes: impl Into<Box<[u8]>>) -> *mut NimmdBuffer {
    let slice = bytes.into();
    let len = slice.len();
    let data = if len == 0 {
        ptr::null_mut()
    } else {
        Box::into_raw(slice).cast::<u8>()
    };
    Box::into_raw(Box::new(NimmdBuffer { data, len }))
}

fn run_on_ffi_stack<F, T>(work: F) -> Result<T, ()>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let join = std::thread::Builder::new()
        .name("nimmd-ffi".into())
        .stack_size(FFI_STACK_BYTES)
        .spawn(move || catch_unwind(AssertUnwindSafe(work)))
        .map_err(|_| ())?;
    match join.join() {
        Ok(Ok(value)) => Ok(value),
        _ => Err(()),
    }
}

unsafe fn copy_bytes(bytes: *const u8, len: usize) -> Vec<u8> {
    if len == 0 || bytes.is_null() {
        return Vec::new();
    }
    // SAFETY: caller обещает `bytes` валиден для `len` байт.
    unsafe { slice::from_raw_parts(bytes, len).to_vec() }
}

unsafe fn parse_options(opts: *const NimmdRenderOptions) -> Option<RenderOptions> {
    if opts.is_null() {
        return Some(RenderOptions::default());
    }
    // SAFETY: caller обещает валидный `NimmdRenderOptions`.
    let raw = unsafe { &*opts };
    if raw.version != NIMMD_RENDER_OPTIONS_VERSION {
        return None;
    }
    let theme = match raw.theme {
        0 => Theme::Auto,
        1 => Theme::Light,
        2 => Theme::Dark,
        _ => return None,
    };
    let math = match raw.math {
        0 => MathMode::Katex,
        1 => MathMode::MathML,
        2 => MathMode::Source,
        _ => return None,
    };
    let assets = match raw.assets {
        0 => AssetMode::None,
        1 => {
            // SAFETY: `base_url` — NULL или валидная C-строка из опций.
            let base = unsafe { c_str(raw.base_url) }.unwrap_or("");
            AssetMode::Link {
                base_url: base.to_owned(),
            }
        }
        _ => return None,
    };
    // SAFETY: `extension` — NULL или валидная C-строка из опций.
    let kind = match unsafe { c_str(raw.extension) } {
        Some(ext) if !ext.is_empty() => FileKind::from_extension(ext).unwrap_or(FileKind::Text),
        _ => FileKind::Markdown,
    };
    let max_bytes = if raw.max_bytes == 0 {
        nimmd_core::text::MAX_BYTES
    } else {
        usize::try_from(raw.max_bytes).ok()?
    };
    Some(RenderOptions {
        theme,
        assets,
        math,
        remote_images: raw.remote_images != 0,
        not_utf8: false,
        max_bytes,
        profile: None,
        kind,
    })
}

unsafe fn optional_path(path: *const c_char) -> Option<PathBuf> {
    // SAFETY: делегирует проверку NULL / UTF-8 в `c_path`.
    unsafe { c_path(path) }
}

unsafe fn c_path(ptr: *const c_char) -> Option<PathBuf> {
    // SAFETY: делегирует проверку NULL / UTF-8 в `c_str`.
    unsafe { c_str(ptr) }.map(PathBuf::from)
}

unsafe fn c_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller обещает NUL-terminated C-строку.
    let c = unsafe { CStr::from_ptr(ptr) };
    c.to_str().ok()
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn buffer_roundtrip_empty_and_bytes() {
        let empty = buffer_from_slice(Vec::<u8>::new());
        assert!(!empty.is_null());
        // SAFETY: только что созданный буфер.
        unsafe {
            assert_eq!((*empty).len, 0);
            assert!((*empty).data.is_null());
            nimmd_buffer_free(empty);
            nimmd_buffer_free(ptr::null_mut());
        }

        let bytes = buffer_from_slice(b"abc".to_vec());
        // SAFETY: только что созданный буфер.
        unsafe {
            assert_eq!((*bytes).len, 3);
            let got = slice::from_raw_parts((*bytes).data, (*bytes).len);
            assert_eq!(got, b"abc");
            nimmd_buffer_free(bytes);
        }
    }

    #[test]
    fn panic_in_render_is_caught_and_returns_null() {
        test_arm_panic_next_render();
        let src = b"# panic please\n";
        let opts = NimmdRenderOptions {
            version: NIMMD_RENDER_OPTIONS_VERSION,
            theme: 0,
            math: 0,
            assets: 0,
            remote_images: 0,
            max_bytes: 0,
            base_url: ptr::null(),
            extension: ptr::null(),
        };
        let opts_ptr = ptr::from_ref(&opts);
        // SAFETY: локальные указатели валидны; паника ловится catch_unwind на потоке 64 МБ.
        let buf = unsafe { nimmd_render_html(src.as_ptr(), src.len(), ptr::null(), opts_ptr) };
        assert!(
            buf.is_null(),
            "паника должна дать NULL, а не сорвать процесс"
        );

        // SAFETY: повторный вызов после снятия флага паники.
        let buf = unsafe { nimmd_render_html(src.as_ptr(), src.len(), ptr::null(), opts_ptr) };
        assert!(!buf.is_null());
        // SAFETY: успешный буфер от render.
        unsafe {
            let html = slice::from_raw_parts((*buf).data, (*buf).len);
            assert!(html.windows(12).any(|w| w == b"panic please"));
            nimmd_buffer_free(buf);
        }
    }
}
