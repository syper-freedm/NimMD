//! Безопасное чтение картинок, на которые ссылается документ
//!
//! `href` — недоверенный текст документа: крейт отдаёт только картинку из канонического
//! каталога документа и в пределах лимитов размера.
#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Предел размера одного файла.
pub const MAX_ASSET_BYTES: u64 = 5 * 1024 * 1024;
/// Предел суммы размеров разных файлов за один бюджет.
pub const MAX_BUDGET_BYTES: u64 = 20 * 1024 * 1024;

const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "webp", "svg"];

/// Бюджет чтения: сумма байт разных файлов и их канонические пути.
///
/// Повторное чтение уже учтённого пути лимит не расходует и ничем не ограничено:
/// хост кэширует байты по пути или сам считает ссылки. Сброс — новый бюджет.
#[derive(Debug, Default)]
pub struct AssetBudget {
    used: u64,
    seen: HashSet<PathBuf>,
}

impl AssetBudget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Сумма байт учтённых файлов.
    pub fn used(&self) -> u64 {
        self.used
    }
}

/// Причина отказа в чтении.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetError {
    /// Битый `%XX`, результат не UTF-8, пустой или с байтом NUL.
    BadEncoding,
    /// Ссылка со схемой (`file:`, `http:`, `C:`).
    Scheme,
    /// Абсолютный путь (начинается с `/` или `\`).
    Absolute,
    /// Путь от домашнего каталога (начинается с `~`).
    Home,
    /// Путь выходит за каталог документа (`..` или симлинк наружу).
    Escape,
    /// Не обычный файл с расширением картинки.
    NotImage,
    /// Файл больше [`MAX_ASSET_BYTES`].
    TooLarge,
    /// Новый файл превысил бы [`MAX_BUDGET_BYTES`].
    BudgetExceeded,
    /// Ошибка файловой системы: нет файла, нет доступа, ошибка чтения.
    Io,
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BadEncoding => "некорректное кодирование ссылки",
            Self::Scheme => "ссылка со схемой не читается",
            Self::Absolute => "абсолютный путь не читается",
            Self::Home => "путь от домашнего каталога не читается",
            Self::Escape => "путь выходит за каталог документа",
            Self::NotImage => "файл не картинка",
            Self::TooLarge => "картинка больше 5 МБ",
            Self::BudgetExceeded => "превышен лимит 20 МБ картинок документа",
            Self::Io => "ошибка чтения файла",
        })
    }
}

impl std::error::Error for AssetError {}

/// Читает картинку `href` относительно каталога документа `doc_path`.
///
/// Байты отдаются как есть: SVG — активное содержимое, хост показывает его только
/// как картинку (`<img>`, data URI), а не вставляет в DOM.
pub fn read_asset(
    doc_path: &Path,
    href: &str,
    budget: &mut AssetBudget,
) -> Result<Vec<u8>, AssetError> {
    let href = decode(href)?;
    if has_scheme(&href) {
        return Err(AssetError::Scheme);
    }
    if href.starts_with(['/', '\\']) {
        return Err(AssetError::Absolute);
    }
    if href.starts_with('~') {
        return Err(AssetError::Home);
    }
    let parts = normalize(&href)?;

    let doc = fs::canonicalize(doc_path).map_err(|_| AssetError::Io)?;
    let base = doc.parent().ok_or(AssetError::Io)?;
    let mut candidate = base.to_path_buf();
    candidate.extend(parts);
    let path = fs::canonicalize(&candidate).map_err(|_| AssetError::Io)?;
    if !path.starts_with(base) {
        return Err(AssetError::Escape);
    }

    let is_image = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| IMAGE_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)));
    if !is_image {
        return Err(AssetError::NotImage);
    }
    let meta = fs::metadata(&path).map_err(|_| AssetError::Io)?;
    if !meta.is_file() {
        return Err(AssetError::NotImage);
    }
    if meta.len() > MAX_ASSET_BYTES {
        return Err(AssetError::TooLarge);
    }
    let known = budget.seen.contains(&path);
    if !known && budget.used.saturating_add(meta.len()) > MAX_BUDGET_BYTES {
        return Err(AssetError::BudgetExceeded);
    }

    let mut bytes = Vec::new();
    File::open(&path)
        .and_then(|f| f.take(MAX_ASSET_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| AssetError::Io)?;
    let read = bytes.len() as u64;
    if read > MAX_ASSET_BYTES {
        return Err(AssetError::TooLarge);
    }
    if !known {
        budget.used += read;
        budget.seen.insert(path);
    }
    Ok(bytes)
}

/// Однократный URL-decode `%XX`.
fn decode(href: &str) -> Result<String, AssetError> {
    let mut out = Vec::with_capacity(href.len());
    let mut bytes = href.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next().and_then(hex).ok_or(AssetError::BadEncoding)?;
            let lo = bytes.next().and_then(hex).ok_or(AssetError::BadEncoding)?;
            out.push(hi << 4 | lo);
        } else {
            out.push(b);
        }
    }
    let s = String::from_utf8(out).map_err(|_| AssetError::BadEncoding)?;
    if s.is_empty() || s.contains('\0') {
        return Err(AssetError::BadEncoding);
    }
    Ok(s)
}

fn hex(b: u8) -> Option<u8> {
    char::from(b)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

/// `[A-Za-z][A-Za-z0-9+.-]*:` в начале строки.
fn has_scheme(s: &str) -> bool {
    s.split_once(':').is_some_and(|(scheme, _)| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    })
}

/// Компоненты пути по `/` и `\` без `.` и пустых; `..` снимает предыдущий.
fn normalize(href: &str) -> Result<Vec<&str>, AssetError> {
    let mut parts = Vec::new();
    for part in href.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or(AssetError::Escape)?;
            }
            _ => parts.push(part),
        }
    }
    Ok(parts)
}
