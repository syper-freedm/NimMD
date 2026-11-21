use std::ffi::{CStr, c_char};
use std::ptr;

use nimmd_core::profile::{Dialect, Features, Profile};
use nimmd_core::text::MAX_BYTES;
use nimmd_core::{AssetMode, FileKind, MathMode, RenderOptions, Theme};

/// Номер версии для структуры [`NimmdRenderOptions`].
pub const NIMMD_RENDER_OPTIONS_V1: u32 = 1;

/// Тема по настройкам системы.
pub const NIMMD_THEME_AUTO: u8 = 0;
/// Светлая тема.
pub const NIMMD_THEME_LIGHT: u8 = 1;
/// Тёмная тема.
pub const NIMMD_THEME_DARK: u8 = 2;

/// Без внешних ресурсов.
pub const NIMMD_ASSETS_NONE: u8 = 0;
/// Ссылки на ресурсы.
pub const NIMMD_ASSETS_LINK: u8 = 1;
/// Ресурсы внутри HTML.
pub const NIMMD_ASSETS_INLINE: u8 = 2;

/// Формулы с KaTeX.
pub const NIMMD_MATH_KATEX: u8 = 0;
/// Формулы в MathML.
pub const NIMMD_MATH_MATHML: u8 = 1;
/// Исходный TeX.
pub const NIMMD_MATH_SOURCE: u8 = 2;

/// Markdown.
pub const NIMMD_KIND_MARKDOWN: u8 = 0;
/// TOML.
pub const NIMMD_KIND_TOML: u8 = 1;
/// YAML.
pub const NIMMD_KIND_YAML: u8 = 2;
/// JSON.
pub const NIMMD_KIND_JSON: u8 = 3;
/// CSV.
pub const NIMMD_KIND_CSV: u8 = 4;
/// Обычный текст.
pub const NIMMD_KIND_TEXT: u8 = 5;

/// Диалект `CommonMark`.
pub const NIMMD_DIALECT_COMMONMARK: u8 = 0;
/// GitHub Flavored Markdown.
pub const NIMMD_DIALECT_GFM: u8 = 1;
/// Диалект `MultiMarkdown`.
pub const NIMMD_DIALECT_MULTIMARKDOWN: u8 = 2;
/// Диалект Obsidian.
pub const NIMMD_DIALECT_OBSIDIAN: u8 = 3;

/// Размещение ресурсов KaTeX и Mermaid. Строки в UTF-8 с завершающим `NUL`.
#[repr(C)]
pub struct NimmdAssets {
    /// Одно из значений `NIMMD_ASSETS_*`.
    pub kind: u8,
    /// Базовый URL без завершающего `/`, только для `NIMMD_ASSETS_LINK`.
    pub base_url: *const c_char,
    /// CSS KaTeX, только для `NIMMD_ASSETS_INLINE`.
    pub katex_css: *const c_char,
    /// JS KaTeX, только для `NIMMD_ASSETS_INLINE`.
    pub katex_js: *const c_char,
    /// JS Mermaid, только для `NIMMD_ASSETS_INLINE`.
    pub mermaid_js: *const c_char,
}

/// Диалект Markdown и явные настройки его расширений.
#[repr(C)]
#[allow(clippy::struct_excessive_bools)] // независимые флаги из nimmd-core
pub struct NimmdProfile {
    /// Одно из значений `NIMMD_DIALECT_*`.
    pub dialect: u8,
    /// YAML перед телом документа.
    pub frontmatter: bool,
    /// Формулы.
    pub math: bool,
    /// Диаграммы Mermaid.
    pub mermaid: bool,
    /// Сноски.
    pub footnotes: bool,
    /// Таблицы.
    pub tables: bool,
    /// Списки задач.
    pub tasklists: bool,
    /// Зачёркивание.
    pub strikethrough: bool,
    /// Вики-ссылки.
    pub wikilinks: bool,
    /// Алерты.
    pub callouts: bool,
}

/// Настройки одного вызова рендера в раскладке версии 1.
///
/// Тип результата `nimmd_render_options_v1_default` закреплён. Для другой
/// раскладки нужна новая структура и отдельная функция инициализации.
#[repr(C)]
pub struct NimmdRenderOptions {
    /// Версия структуры, которую передаёт вызывающий код.
    pub version: u32,
    /// Одно из значений `NIMMD_THEME_*`.
    pub theme: u8,
    /// Как подключить KaTeX и Mermaid.
    pub assets: NimmdAssets,
    /// Одно из значений `NIMMD_MATH_*`.
    pub math: u8,
    /// Разрешить удалённые ссылки на картинки в HTML. Для Quick Look — `false`.
    pub remote_images: bool,
    /// Показывать предупреждение о замене невалидных последовательностей UTF-8.
    pub not_utf8: bool,
    /// Предел исходного текста в байтах, не выше [`MAX_BYTES`].
    pub max_bytes: usize,
    /// Явный профиль или `NULL` для автоопределения.
    pub profile: *const NimmdProfile,
    /// Одно из значений `NIMMD_KIND_*`.
    pub kind: u8,
}

impl NimmdRenderOptions {
    pub(crate) const fn defaults() -> Self {
        Self {
            version: NIMMD_RENDER_OPTIONS_V1,
            theme: NIMMD_THEME_AUTO,
            assets: NimmdAssets {
                kind: NIMMD_ASSETS_NONE,
                base_url: ptr::null(),
                katex_css: ptr::null(),
                katex_js: ptr::null(),
                mermaid_js: ptr::null(),
            },
            math: NIMMD_MATH_KATEX,
            remote_images: false,
            not_utf8: false,
            max_bytes: MAX_BYTES,
            profile: ptr::null(),
            kind: NIMMD_KIND_MARKDOWN,
        }
    }
}

/// # Safety
/// Для ненулевого указателя действителен контракт `nimmd_render_html`.
pub(crate) unsafe fn read(options: *const NimmdRenderOptions) -> Option<RenderOptions> {
    if options.is_null() {
        return Some(RenderOptions::default());
    }
    // SAFETY: начальное поле version доступно даже у структуры другой версии.
    if unsafe { options.cast::<u32>().read() } != NIMMD_RENDER_OPTIONS_V1 {
        return None;
    }
    // SAFETY: версия совпадает. Хост держит доступную полную структуру.
    let options = unsafe { &*options };
    let theme = match options.theme {
        NIMMD_THEME_AUTO => Theme::Auto,
        NIMMD_THEME_LIGHT => Theme::Light,
        NIMMD_THEME_DARK => Theme::Dark,
        _ => return None,
    };
    let math = match options.math {
        NIMMD_MATH_KATEX => MathMode::Katex,
        NIMMD_MATH_MATHML => MathMode::MathML,
        NIMMD_MATH_SOURCE => MathMode::Source,
        _ => return None,
    };
    let kind = match options.kind {
        NIMMD_KIND_MARKDOWN => FileKind::Markdown,
        NIMMD_KIND_TOML => FileKind::Toml,
        NIMMD_KIND_YAML => FileKind::Yaml,
        NIMMD_KIND_JSON => FileKind::Json,
        NIMMD_KIND_CSV => FileKind::Csv,
        NIMMD_KIND_TEXT => FileKind::Text,
        _ => return None,
    };
    // SAFETY: ненулевой указатель профиля по контракту действителен до возврата.
    let profile = match unsafe { options.profile.as_ref() } {
        Some(profile) => Some(profile.to_core()?),
        None => None,
    };
    let assets = &options.assets;
    let assets = match assets.kind {
        NIMMD_ASSETS_NONE => AssetMode::None,
        // SAFETY: контракт ABI гарантирует строку с NUL, если указатель ненулевой.
        NIMMD_ASSETS_LINK => unsafe {
            AssetMode::Link {
                base_url: string(assets.base_url)?,
            }
        },
        // SAFETY: контракт ABI гарантирует строки с NUL, если указатели ненулевые.
        NIMMD_ASSETS_INLINE => unsafe {
            AssetMode::Inline {
                katex_css: string(assets.katex_css)?,
                katex_js: string(assets.katex_js)?,
                mermaid_js: string(assets.mermaid_js)?,
            }
        },
        _ => return None,
    };
    Some(RenderOptions {
        theme,
        assets,
        math,
        remote_images: options.remote_images,
        not_utf8: options.not_utf8,
        max_bytes: options.max_bytes.min(MAX_BYTES),
        profile,
        kind,
    })
}

unsafe fn string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: вызывающий код хранит переданную C-строку до получения копии.
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

impl NimmdProfile {
    fn to_core(&self) -> Option<Profile> {
        let dialect = match self.dialect {
            NIMMD_DIALECT_COMMONMARK => Dialect::CommonMark,
            NIMMD_DIALECT_GFM => Dialect::Gfm,
            NIMMD_DIALECT_MULTIMARKDOWN => Dialect::MultiMarkdown,
            NIMMD_DIALECT_OBSIDIAN => Dialect::Obsidian,
            _ => return None,
        };
        Some(Profile {
            dialect,
            features: Features {
                frontmatter: self.frontmatter,
                math: self.math,
                mermaid: self.mermaid,
                footnotes: self.footnotes,
                tables: self.tables,
                tasklists: self.tasklists,
                strikethrough: self.strikethrough,
                wikilinks: self.wikilinks,
                callouts: self.callouts,
            },
        })
    }
}
