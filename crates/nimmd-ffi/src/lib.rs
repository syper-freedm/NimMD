//! C ABI ядра NimMD и чтения локальных картинок.
#![warn(missing_docs)]

use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::{Path, PathBuf};
use std::{mem, ptr, slice, str, thread};

use nimmd_assets::{AssetBudget, AssetDirectory, Image};
use nimmd_core::text;

mod options;

pub use options::*;

/// Выделенный Rust буфер. Освобождается через [`nimmd_buffer_free`].
#[repr(C)]
pub struct NimmdBuffer {
    /// Указатель на начало буфера или `NULL` при отказе чтения картинки.
    pub ptr: *mut u8,
    /// Размер буфера в байтах. Терминатора `NUL` нет.
    pub len: usize,
}

impl NimmdBuffer {
    fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::empty();
        }
        let len = bytes.len();
        let ptr = Box::into_raw(bytes.into_boxed_slice()).cast::<u8>();
        Self { ptr, len }
    }
}

/// Результат чтения картинки с типом конечного файла.
#[repr(C)]
pub struct NimmdImage {
    /// Байты картинки. Освобождаются через [`nimmd_buffer_free`].
    pub buffer: NimmdBuffer,
    /// Статическая строка MIME с `NUL` или `NULL` при отказе.
    pub content_type: *const c_char,
}

impl NimmdImage {
    fn empty() -> Self {
        Self {
            buffer: NimmdBuffer::empty(),
            content_type: ptr::null(),
        }
    }

    fn from_image(image: Image) -> Option<Self> {
        if image.bytes.is_empty() {
            return None;
        }
        let content_type = match image.content_type {
            "image/png" => c"image/png",
            "image/jpeg" => c"image/jpeg",
            "image/gif" => c"image/gif",
            "image/webp" => c"image/webp",
            "image/svg+xml" => c"image/svg+xml",
            _ => return None,
        };
        Some(Self {
            buffer: NimmdBuffer::from_bytes(image.bytes),
            content_type: content_type.as_ptr(),
        })
    }
}

/// Непрозрачный бюджет картинок одного документа с закреплённым каталогом.
pub struct NimmdAssetBudget {
    document: PathBuf,
    directory: AssetDirectory,
    budget: AssetBudget,
}

impl NimmdAssetBudget {
    fn open(document: &Path) -> Option<Self> {
        if document.canonicalize().ok()? != document {
            return None;
        }
        Some(Self {
            document: document.to_owned(),
            directory: AssetDirectory::open(document.parent()?).ok()?,
            budget: AssetBudget::new(),
        })
    }
}

/// Оформляет документ и возвращает HTML в UTF-8.
///
/// Пустой документ допускает `ptr == NULL`. Для опций по умолчанию передайте
/// `options == NULL`. При ошибке опций, запуска потока или панике вернётся HTML с ошибкой.
///
/// # Safety
///
/// Ненулевой `ptr` указывает на `len` инициализированных байт. Ненулевой `options`
/// указывает на выровненный `uint32_t` с версией. Для поддерживаемой версии он
/// указывает на полностью инициализированную и выровненную структуру с валидными
/// полями, а задействованные вложенные указатели — на соответствующие структуры
/// и строки с `NUL`. Входные данные должны быть доступны и неизменны до возврата.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_render_html(
    ptr: *const u8,
    len: usize,
    options: *const NimmdRenderOptions,
) -> NimmdBuffer {
    render_boundary(|| {
        // SAFETY: контракт вызова обеспечивает доступ к опциям и вложенным указателям.
        let mut options = unsafe { options::read(options) }?;
        let bytes = if len == 0 {
            &[]
        } else {
            if ptr.is_null() || len > isize::MAX as usize {
                return None;
            }
            // CRLF и BOM учитываются ядром до обрезки. Одного запаса хватает и на хвост UTF-8.
            let limit = 2 * options.max_bytes + 8;
            // SAFETY: ptr проверен. Хост держит len байт, а мы берём только их префикс.
            let prefix = unsafe { slice::from_raw_parts(ptr, len.min(limit)) };
            if len > limit {
                trim_incomplete_tail(prefix)
            } else {
                prefix
            }
        };
        render_on_worker(move || {
            let (source, not_utf8) = text::decode_lossy(bytes);
            options.not_utf8 |= not_utf8;
            nimmd_core::render(&source, &options).html
        })
    })
}

fn trim_incomplete_tail(bytes: &[u8]) -> &[u8] {
    let tail = bytes
        .utf8_chunks()
        .last()
        .map_or(&[][..], |chunk| chunk.invalid());
    if str::from_utf8(tail).is_err_and(|error| error.error_len().is_none()) {
        &bytes[..bytes.len() - tail.len()]
    } else {
        bytes
    }
}

fn render_on_worker(render: impl FnOnce() -> String + Send) -> Option<String> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, render)
            .ok()?;
        match worker.join() {
            Ok(html) => Some(html),
            Err(payload) => resume_unwind(payload),
        }
    })
}

fn render_boundary(render: impl FnOnce() -> Option<String>) -> NimmdBuffer {
    guard(render_error, || {
        render().map_or_else(render_error, |html| {
            NimmdBuffer::from_bytes(html.into_bytes())
        })
    })
}

fn render_error() -> NimmdBuffer {
    NimmdBuffer::from_bytes(
        "<!doctype html><meta charset=\"utf-8\"><p>Не удалось показать документ</p>"
            .as_bytes()
            .to_vec(),
    )
}

fn guard<T>(fallback: impl FnOnce() -> T, call: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(value) => value,
        Err(payload) => {
            // Деструктор произвольного объекта паники сам может вызвать панику.
            if let Err(drop_panic) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                mem::forget(drop_panic);
            }
            fallback()
        }
    }
}

/// Стандартные опции рендера версии 1. Меняйте поля в копии перед вызовом ABI.
#[unsafe(no_mangle)]
pub extern "C" fn nimmd_render_options_v1_default() -> NimmdRenderOptions {
    guard(NimmdRenderOptions::defaults, NimmdRenderOptions::defaults)
}

/// Закрепляет каталог существующего документа и выделяет бюджет на один рендер.
/// При ошибке, в том числе для неканонического пути, возвращает `NULL`.
///
/// # Safety
///
/// Ненулевой `doc_path` указывает на доступный до возврата канонический путь
/// в UTF-8 с `NUL`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_asset_budget_open(doc_path: *const c_char) -> *mut NimmdAssetBudget {
    guard(ptr::null_mut, || {
        if doc_path.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: ненулевой указатель по контракту ведёт к доступной строке с NUL.
        unsafe { CStr::from_ptr(doc_path) }
            .to_str()
            .ok()
            .and_then(|path| NimmdAssetBudget::open(Path::new(path)))
            .map_or_else(ptr::null_mut, |budget| Box::into_raw(Box::new(budget)))
    })
}

/// Читает локальную картинку из каталога документа в пределах бюджета.
///
/// При отказе возвращает пустой буфер. SVG можно показывать в контексте картинки.
///
/// # Safety
///
/// Каждый ненулевой указатель строки указывает на доступную до возврата строку
/// с `NUL`. Пути в UTF-8. Ненулевой `budget` получен из [`nimmd_asset_budget_open`].
/// Передавайте тот же путь `doc_path`, что при открытии бюджета. Вызовы с этим
/// бюджетом последовательны.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_read_asset(
    doc_path: *const c_char,
    href: *const c_char,
    budget: *mut NimmdAssetBudget,
) -> NimmdBuffer {
    guard(NimmdBuffer::empty, || {
        // SAFETY: действуют условия указателей и бюджета текущего вызова ABI.
        unsafe { read_image(doc_path, href, budget) }.map_or_else(NimmdBuffer::empty, |image| {
            NimmdBuffer::from_bytes(image.bytes)
        })
    })
}

/// Читает локальную картинку и возвращает её байты и MIME. Строка MIME
/// остаётся доступной после освобождения буфера картинки.
///
/// # Safety
///
/// Условия указателей и бюджета совпадают с [`nimmd_read_asset`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_read_asset_with_type(
    doc_path: *const c_char,
    href: *const c_char,
    budget: *mut NimmdAssetBudget,
) -> NimmdImage {
    guard(NimmdImage::empty, || {
        // SAFETY: действуют условия указателей и бюджета текущего вызова ABI.
        unsafe { read_image(doc_path, href, budget) }
            .and_then(NimmdImage::from_image)
            .unwrap_or_else(NimmdImage::empty)
    })
}

unsafe fn read_image(
    doc_path: *const c_char,
    href: *const c_char,
    budget: *mut NimmdAssetBudget,
) -> Option<Image> {
    if doc_path.is_null() || href.is_null() || budget.is_null() {
        return None;
    }
    // SAFETY: оба указателя ненулевые и по контракту ведут к строкам с NUL.
    let (doc_path, href) = unsafe { (CStr::from_ptr(doc_path), CStr::from_ptr(href)) };
    let doc_path = Path::new(doc_path.to_str().ok()?);
    let href = href.to_str().ok()?;
    // SAFETY: хост держит действительный бюджет и обращается к нему последовательно.
    let state = unsafe { &mut *budget };
    if doc_path != state.document {
        return None;
    }
    state.directory.read(href, &mut state.budget).ok()
}

/// Освобождает бюджет картинок. Принимает `NULL`.
///
/// # Safety
///
/// Ненулевой `budget` выделен [`nimmd_asset_budget_open`], ещё не освобождён
/// и больше не используется хостом или другим вызовом ABI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_asset_budget_free(budget: *mut NimmdAssetBudget) {
    guard(
        || (),
        || {
            if !budget.is_null() {
                // SAFETY: по контракту это действительный Box, владение возвращается один раз.
                drop(unsafe { Box::from_raw(budget) });
            }
        },
    );
}

/// Освобождает буфер NimMD. Пустой буфер допустим.
///
/// # Safety
///
/// Буфер получен из ABI NimMD. Его указатель и размер сохранены, а буфер ещё
/// не освобождён и больше не читается хостом.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nimmd_buffer_free(buffer: NimmdBuffer) {
    guard(
        || (),
        || {
            if !buffer.ptr.is_null() {
                let slice = ptr::slice_from_raw_parts_mut(buffer.ptr, buffer.len);
                // SAFETY: по контракту ptr и len принадлежат исходному Box<[u8]>.
                drop(unsafe { Box::from_raw(slice) });
            }
        },
    );
}

#[cfg(test)]
mod tests;
