use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use nimmd_core::{AssetMode, FileKind, RenderOptions, Theme, render, text};
use same_file::Handle;
use sha2::{Digest, Sha256};

// Запас на CRLF, BOM и многобайтовый символ у границы нормализованного текста.
const MAX_READ: usize = 2 * text::MAX_BYTES + 8;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub(super) struct SourceFormat {
    pub(super) bom: bool,
    pub(super) crlf: bool,
}

impl SourceFormat {
    pub(super) fn detect(source: &str) -> Self {
        Self {
            bom: text::strip_bom(source).0,
            crlf: text::detect_line_ending(source) == text::LineEnding::CrLf,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
enum ReadOnlyReason {
    TooLarge,
    NotUtf8,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EditorDocument {
    pub(super) body: String,
    pub(super) source: Option<String>,
    revision: Option<String>,
    format: Option<SourceFormat>,
    read_only_reason: Option<ReadOnlyReason>,
    read_only_text: Option<String>,
}

#[derive(Clone)]
pub(super) struct DocumentFile {
    path: PathBuf,
    #[cfg(target_os = "macos")]
    canonical: Option<PathBuf>,
    identity: Result<Arc<Handle>, String>,
}

impl DocumentFile {
    pub(super) fn prepare(
        path: &Path,
        canonical_file: impl FnOnce() -> io::Result<PathBuf>,
    ) -> io::Result<(Option<PathBuf>, Self)> {
        // Дескриптор удерживает идентификатор выбранного файла даже после удаления.
        let (key, identity) = match open_regular(path) {
            Ok(original) => match canonical_file().and_then(|canonical| {
                open_matching(&canonical, &original)?;
                Ok(canonical)
            }) {
                Ok(canonical) => (Some(canonical), Ok(Arc::new(original))),
                Err(error) => (None, Err(error)),
            },
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                // Для поиска окна достаточно пути. Без дескриптора просмотр вернёт ошибку.
                (canonical_file().ok(), Err(error))
            }
            Err(error) => {
                canonical_file()?;
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::InvalidInput
                ) {
                    return Err(io::Error::new(error.kind(), read_error(path, &error)));
                }
                (None, Err(error))
            }
        };
        Ok((
            key.clone(),
            Self {
                #[cfg(target_os = "macos")]
                canonical: key,
                path: path.to_owned(),
                identity: identity.map_err(|error| read_error(path, &error)),
            },
        ))
    }

    pub(super) fn render(&self) -> Result<String, String> {
        self.read(false).map(|document| document.body)
    }

    pub(super) fn editor_document(&self) -> Result<EditorDocument, String> {
        #[cfg(target_os = "macos")]
        let _operation = super::save::disk_edit();
        self.read(true)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn editor_path(&self) -> Result<&Path, String> {
        self.canonical.as_deref().ok_or_else(|| {
            "Не удалось определить путь к документу. Откройте файл снова.".to_owned()
        })
    }

    #[cfg(target_os = "macos")]
    pub(super) fn editor_latest(&self) -> Result<EditorDocument, String> {
        let path = self.editor_path()?;
        let read = || {
            let _operation = super::save::disk_edit();
            if path.canonicalize()? != path {
                return Err(io::Error::other("путь к документу изменился"));
            }
            let file = super::save::open_regular(path, rustix::fs::OFlags::RDONLY)?;
            self.render_file(&file, true, false)
        };
        read().map_err(|error| read_error(&self.path, &error))
    }

    fn read(&self, include_source: bool) -> Result<EditorDocument, String> {
        let original = self.identity.as_ref().map_err(Clone::clone)?;
        self.render_bound(original, include_source)
            .map_err(|error| read_error(&self.path, &error))
    }

    fn render_bound(&self, original: &Handle, include_source: bool) -> io::Result<EditorDocument> {
        let current = open_matching(&self.path, original)?;
        self.render_file(current.as_file(), include_source, true)
    }

    fn render_file(
        &self,
        file: &File,
        include_source: bool,
        include_body: bool,
    ) -> io::Result<EditorDocument> {
        let mut bytes = Vec::new();
        let limit = if include_source {
            text::MAX_BYTES + 4
        } else {
            MAX_READ
        };
        file.take(limit as u64).read_to_end(&mut bytes)?;
        let too_large = bytes.len() > text::MAX_BYTES;
        let incomplete = bytes.len() == limit && file.metadata()?.len() > limit as u64;
        if incomplete {
            let tail = bytes
                .utf8_chunks()
                .last()
                .map_or(&[][..], |chunk| chunk.invalid());
            if std::str::from_utf8(tail).is_err_and(|error| error.error_len().is_none()) {
                bytes.truncate(limit - tail.len());
            }
        }

        let (source, not_utf8) = text::decode_lossy(&bytes);
        if include_source && (too_large || not_utf8) {
            let text = text::strip_bom(&source).1;
            return Ok(EditorDocument {
                body: String::new(),
                source: None,
                revision: (!too_large).then(|| revision(&bytes)),
                format: None,
                read_only_reason: Some(if too_large {
                    ReadOnlyReason::TooLarge
                } else {
                    ReadOnlyReason::NotUtf8
                }),
                read_only_text: Some(
                    text::truncate_at_char_boundary(text, text::MAX_BYTES)
                        .0
                        .to_owned(),
                ),
            });
        }
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
        let body = if include_body {
            render(&source, &options).body
        } else {
            String::new()
        };
        let format = include_source.then(|| SourceFormat::detect(&source));
        let source = include_source.then(|| text::strip_bom(&source).1.to_owned());
        let revision = include_source.then(|| revision(&bytes));
        Ok(EditorDocument {
            revision,
            format,
            read_only_reason: None,
            read_only_text: None,
            body,
            source,
        })
    }
}

pub(super) fn revision(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn open_matching(path: &Path, original: &Handle) -> io::Result<Handle> {
    let current = open_regular(path)?;
    if &current != original {
        return Err(io::Error::other(
            "по этому пути теперь находится другой файл",
        ));
    }
    Ok(current)
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

    fn bind(path: &Path) -> DocumentFile {
        let (key, view) = DocumentFile::prepare(path, || path.canonicalize()).unwrap();
        assert!(key.is_some());
        view
    }

    #[test]
    fn editor_gets_body_and_original_text_from_one_read() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.md");
        let source = "# Заголовок\r\n\r\n%%неизвестно%%\rстрока\n";
        std::fs::write(&path, format!("\u{FEFF}{source}")).unwrap();

        let document = bind(&path).editor_document().unwrap();
        assert_eq!(document.source.as_deref(), Some(source));
        assert!(document.body.contains("<h1>Заголовок</h1>"));
    }

    #[test]
    fn editor_does_not_get_lossy_or_truncated_source() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.txt");
        std::fs::write(&path, b"text\xff").unwrap();
        let document = bind(&path).editor_document().unwrap();
        assert!(document.source.is_none());
        assert_eq!(document.read_only_reason, Some(ReadOnlyReason::NotUtf8));
        assert_eq!(document.read_only_text.as_deref(), Some("text\u{fffd}"));
        assert_eq!(document.body, "");

        std::fs::write(&path, "x".repeat(text::MAX_BYTES + 1)).unwrap();
        let document = bind(&path).editor_document().unwrap();
        assert!(document.source.is_none());
        assert_eq!(document.read_only_reason, Some(ReadOnlyReason::TooLarge));
        assert_eq!(
            document.read_only_text.as_deref().map(str::len),
            Some(text::MAX_BYTES)
        );
        assert_eq!(document.body, "");
    }

    #[test]
    fn editor_uses_raw_byte_limit_before_normalizing_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        let text_at_limit = "x\r\n".repeat(text::MAX_BYTES / 3) + &"x".repeat(text::MAX_BYTES % 3);
        std::fs::write(&path, format!("{text_at_limit}x")).unwrap();
        let doc = bind(&path).editor_document().unwrap();
        assert!(doc.source.is_none());
        assert_eq!(doc.read_only_reason, Some(ReadOnlyReason::TooLarge));
        assert_eq!(doc.read_only_text.as_deref(), Some(text_at_limit.as_str()));
        assert_eq!(doc.body, "");
    }

    #[test]
    fn reports_lossy_text_from_the_bound_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("данные.TXT");
        std::fs::write(&path, b"\xef\xbb\xbf<literal>\r\n\xff").unwrap();
        let body = bind(&path).render().unwrap();
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
        let original = bind(&path);
        std::fs::remove_file(&path).unwrap();
        let missing = original.render().unwrap_err();
        assert!(missing.contains("«заметка.md»"));
        assert!(missing.contains("закройте окно и откройте его снова"));
        std::fs::write(&path, "# Другая заметка").unwrap();
        assert!(original.render().is_err());
        assert!(bind(&path).render().unwrap().contains("Другая заметка"));
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
        let bound = bind(&path);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        assert!(bound.render().unwrap_err().contains("«недоступный.md»"));
        let (_, denied) = DocumentFile::prepare(&path, || path.canonicalize()).unwrap();
        assert!(denied.render().unwrap_err().contains("«недоступный.md»"));
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
        let bound = bind(&alias);
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
        let bound = bind(&path);
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let (key, preparing) = DocumentFile::prepare(&path, || {
                let canonical = path.canonicalize()?;
                std::fs::rename(&path, folder.path().join("исходник.txt"))?;
                assert!(
                    std::process::Command::new("mkfifo")
                        .arg(&path)
                        .status()?
                        .success()
                );
                Ok(canonical)
            })
            .unwrap();
            assert!(key.is_none());
            for view in [preparing, bound] {
                let error = view.render().unwrap_err();
                assert!(error.contains("«file.txt»"));
                assert!(error.contains("нужен обычный файл"));
            }
            tx.send(()).unwrap();
        });
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn rejects_replacing_the_file_during_preparation() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.md");
        std::fs::write(&path, "# Исходник").unwrap();
        let (key, view) = DocumentFile::prepare(&path, || {
            let canonical = path.canonicalize()?;
            std::fs::rename(&path, folder.path().join("исходник.md"))?;
            std::fs::write(&path, "# Замена")?;
            Ok(canonical)
        })
        .unwrap();
        let error = view.render().unwrap_err();
        assert!(error.contains("«заметка.md»"));
        assert!(error.contains("по этому пути теперь находится другой файл"));
        assert!(key.is_none());
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
        let bound = bind(&path);
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
