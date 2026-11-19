use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use nimmd_core::{AssetMode, FileKind, RenderOptions, Theme, render, text};
use same_file::Handle;

// Запас на CRLF, BOM и многобайтовый символ у границы нормализованного текста.
const MAX_READ: usize = 2 * text::MAX_BYTES + 8;

#[derive(Clone)]
pub(super) struct ViewFile {
    path: PathBuf,
    identity: Result<Arc<Handle>, String>,
}

impl ViewFile {
    pub(super) fn bind(path: PathBuf, canonical: &Path) -> Self {
        // Открытый дескриптор удерживает исходный идентификатор даже после удаления файла.
        let identity = open_regular(canonical)
            .map(Arc::new)
            .map_err(|error| read_error(&path, &error));
        Self { path, identity }
    }

    pub(super) fn render(&self) -> Result<String, String> {
        let original = self.identity.as_ref().map_err(Clone::clone)?;
        self.render_bound(original)
            .map_err(|error| read_error(&self.path, &error))
    }

    fn render_bound(&self, original: &Handle) -> io::Result<String> {
        let current = open_regular(&self.path)?;
        if &current != original {
            return Err(io::Error::other(
                "по этому пути теперь находится другой файл",
            ));
        }

        let file = current.as_file();
        let mut bytes = Vec::new();
        file.take(MAX_READ as u64).read_to_end(&mut bytes)?;
        if bytes.len() == MAX_READ && file.metadata()?.len() > MAX_READ as u64 {
            let tail = bytes
                .utf8_chunks()
                .last()
                .map_or(&[][..], |chunk| chunk.invalid());
            if std::str::from_utf8(tail).is_err_and(|error| error.error_len().is_none()) {
                bytes.truncate(MAX_READ - tail.len());
            }
        }

        let (source, not_utf8) = text::decode_lossy(&bytes);
        let kind = self
            .path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(FileKind::from_extension)
            .unwrap_or_default();
        let options = RenderOptions {
            theme: Theme::Auto,
            remote_images: false,
            assets: AssetMode::Link {
                base_url: "/vendor".into(),
            },
            kind,
            not_utf8,
            ..RenderOptions::default()
        };
        Ok(render(&source, &options).body)
    }
}

fn open_regular(path: &Path) -> io::Result<Handle> {
    #[cfg(unix)]
    let file = {
        use rustix::fs::{Mode, OFlags, open};

        File::from(open(
            path,
            OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOCTTY | OFlags::CLOEXEC,
            Mode::empty(),
        )?)
    };
    #[cfg(not(unix))]
    let file = File::open(path)?;

    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "нужен обычный файл",
        ));
    }
    Handle::from_file(file)
}

fn read_error(path: &Path, error: &io::Error) -> String {
    let name = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy();
    format!(
        "Не удалось прочитать «{name}» ({error}). Проверьте доступ к файлу, закройте окно и откройте его снова."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_lossy_text_from_the_bound_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("данные.TXT");
        std::fs::write(&path, b"\xef\xbb\xbf<literal>\r\n\xff").unwrap();
        let body = ViewFile::bind(path.clone(), &path).render().unwrap();
        assert!(body.starts_with(r#"<main class="nimmd nimmd-data" data-kind="text">"#));
        assert!(body.contains(r#"class="banner not-utf8""#));
        assert!(body.contains("&lt;literal&gt;\n\u{FFFD}"));
        assert!(!body.contains('\u{FEFF}'));
    }

    #[test]
    fn rejects_replacing_or_removing_the_bound_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.md");
        std::fs::write(&path, "# Исходник").unwrap();
        let original = ViewFile::bind(path.clone(), &path);
        std::fs::remove_file(&path).unwrap();
        let missing = original.render().unwrap_err();
        assert!(missing.contains("«заметка.md»"));
        assert!(missing.contains("закройте окно и откройте его снова"));
        std::fs::write(&path, "# Другая заметка").unwrap();
        assert!(original.render().is_err());
        assert!(
            ViewFile::bind(path.clone(), &path)
                .render()
                .unwrap()
                .contains("Другая заметка")
        );
    }

    #[cfg(unix)]
    #[test]
    fn reports_lost_read_access() {
        use std::os::unix::fs::PermissionsExt;

        if rustix::process::geteuid().is_root() {
            return;
        }
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("недоступный.md");
        std::fs::write(&path, "# Исходник").unwrap();
        let bound = ViewFile::bind(path.clone(), &path);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        assert!(bound.render().unwrap_err().contains("«недоступный.md»"));
    }

    #[cfg(unix)]
    #[test]
    fn reads_symlink_then_rejects_retargeting_it() {
        let folder = tempfile::tempdir().unwrap();
        let original = folder.path().join("file.md");
        let other = folder.path().join("other.md");
        let alias = folder.path().join("ссылка.md");
        std::fs::write(&original, "# Исходник").unwrap();
        std::fs::write(&other, "# Замена").unwrap();
        std::os::unix::fs::symlink(&original, &alias).unwrap();
        let bound = ViewFile::bind(alias.clone(), &original);
        assert!(bound.render().unwrap().contains("Исходник"));
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&other, &alias).unwrap();
        assert!(bound.render().unwrap_err().contains("«ссылка.md»"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_fifo_without_waiting_for_input() {
        use std::{sync::mpsc, thread, time::Duration};

        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("file.txt");
        std::fs::write(&path, "Исходник").unwrap();
        let bound = ViewFile::bind(path.clone(), &path);
        std::fs::remove_file(&path).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || tx.send(bound.render()).unwrap());
        assert!(rx.recv_timeout(Duration::from_secs(2)).unwrap().is_err());
        worker.join().unwrap();
    }

    #[test]
    fn distinguishes_read_limit_from_incomplete_symbol_at_eof() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("большой.txt");
        let mut input = b"\xef\xbb\xbf".to_vec();
        for _ in 0..text::MAX_BYTES - 1 {
            input.extend_from_slice(b"\r\n");
        }
        input.extend_from_slice("€\r\n".as_bytes());
        assert_eq!(input.len(), MAX_READ - 2);
        input.extend_from_slice(&[0xe2, 0x82]);
        std::fs::write(&path, &input).unwrap();
        let bound = ViewFile::bind(path.clone(), &path);
        let exact = bound.render().unwrap();
        assert!(exact.contains(r#"class="banner not-utf8""#));
        assert!(exact.contains(r#"class="banner truncated""#));

        input.push(0xac);
        std::fs::write(&path, &input).unwrap();
        let (source, not_utf8) = text::decode_lossy(&input);
        let expected = render(
            &source,
            &RenderOptions {
                kind: FileKind::Text,
                not_utf8,
                ..RenderOptions::default()
            },
        )
        .body;
        assert_eq!(bound.render().unwrap(), expected);
    }
}
