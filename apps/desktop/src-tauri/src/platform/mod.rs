use std::{io, path::Path};

/// Платформенная граница операций над документом (К17).
pub trait Platform {
    fn copy_metadata(&self, src: &Path, dst: &Path) -> io::Result<()>;
    fn set_document_edited(&self, window: &tauri::WebviewWindow, edited: bool) -> io::Result<()>;
    fn set_represented_file(&self, window: &tauri::WebviewWindow, path: &Path) -> io::Result<()>;
}

/// shortcut: операции возвращают Unsupported, реализовать при добавлении открытия и сохранения файлов.
pub struct NativePlatform;

impl Platform for NativePlatform {
    fn copy_metadata(&self, _src: &Path, _dst: &Path) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Перенос метаданных ещё не реализован",
        ))
    }

    fn set_document_edited(&self, _window: &tauri::WebviewWindow, _edited: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Признак изменения документа ещё не реализован",
        ))
    }

    fn set_represented_file(&self, _window: &tauri::WebviewWindow, _path: &Path) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Иконка документа в заголовке ещё не реализована",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::app::launch::Launch;

    #[test]
    #[cfg(unix)]
    fn rejects_non_utf8_path() {
        use std::os::unix::ffi::OsStringExt;

        assert!(Launch::parse([OsString::from_vec(vec![0xff])]).is_err());
    }

    #[test]
    #[cfg(windows)]
    fn rejects_non_unicode_path() {
        use std::os::windows::ffi::OsStringExt;

        assert!(Launch::parse([OsString::from_wide(&[0xd800])]).is_err());
    }
}
