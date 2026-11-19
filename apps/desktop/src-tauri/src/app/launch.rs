use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Режим окна, выбранный при запуске.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Edit,
    View,
}

/// Параметры запуска; файл пока не читается.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Launch {
    pub mode: Mode,
    pub path: Option<PathBuf>,
}

impl Launch {
    /// Разбирает аргументы без имени исполняемого файла.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> io::Result<Self> {
        let mut launch = Self::default();
        let mut positional_only = false;
        for arg in args {
            let arg = arg.into_string().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Путь должен быть в UTF-8")
            })?;
            match arg.as_str() {
                "--" if !positional_only => positional_only = true,
                "--view" if !positional_only && launch.mode == Mode::Edit => {
                    launch.mode = Mode::View;
                }
                value if !positional_only && value.starts_with('-') => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Неизвестный или повторный параметр: {value}"),
                    ));
                }
                "" => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Пустой путь"));
                }
                value if launch.path.is_none() => launch.path = Some(PathBuf::from(value)),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Укажите один файл: nimmd [--view] [--] файл",
                    ));
                }
            }
        }
        if launch.path.is_none() && (launch.mode == Mode::View || positional_only) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Не указан файл: nimmd [--view] [--] файл",
            ));
        }
        Ok(launch)
    }

    pub(super) fn resolve_cli_path(&mut self) -> io::Result<()> {
        if let Some(path) = &mut self.path {
            if path.is_relative() {
                *path = std::env::current_dir()?.join(&*path);
            }
            check_display_path(path)?;
        }
        Ok(())
    }

    pub(super) fn from_path(path: PathBuf) -> Self {
        Self {
            mode: Mode::Edit,
            path: Some(path),
        }
    }

    pub(super) fn from_url(url: &tauri::Url) -> io::Result<Self> {
        if url.scheme() == "file"
            && url.host_str().is_none_or(|host| host == "localhost")
            && url.query().is_none()
            && url.fragment().is_none()
            && let Ok(path) = url.to_file_path()
        {
            return Ok(Self::from_path(path));
        }
        let name = url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .filter(|name| !name.is_empty())
            .unwrap_or("файл по URL");
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Не удалось открыть «{name}». Выберите локальный файл и откройте его снова."),
        ))
    }

    pub(super) fn canonical_file(&self) -> io::Result<Option<PathBuf>> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        check_display_path(path)?;
        if !path.is_absolute() {
            return Err(file_error(
                path,
                io::ErrorKind::InvalidInput,
                "Укажите абсолютный путь и откройте файл снова.",
            ));
        }
        let unavailable = |error: io::Error| {
            file_error(
                path,
                error.kind(),
                "Проверьте, что файл существует и доступен, затем откройте его снова.",
            )
        };
        let canonical = path.canonicalize().map_err(unavailable)?;
        if !canonical.metadata().map_err(unavailable)?.is_file() {
            return Err(file_error(
                path,
                io::ErrorKind::InvalidInput,
                "Выберите обычный файл и откройте его снова.",
            ));
        }
        Ok(Some(canonical))
    }

    pub fn window_title(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("NimMD")
            .to_owned()
    }
}

fn check_display_path(path: &Path) -> io::Result<()> {
    if path.to_str().is_none() {
        return Err(file_error(
            path,
            io::ErrorKind::InvalidInput,
            "Поддерживаются пути в UTF-8. Переименуйте файл или его папку и откройте его снова.",
        ));
    }
    Ok(())
}

fn file_error(path: &Path, kind: io::ErrorKind, advice: &str) -> io::Error {
    let name = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy();
    io::Error::new(kind, format!("Не удалось открыть «{name}». {advice}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_cli_path_to_callers_directory() {
        let mut launch = Launch::parse([
            OsString::from("--view"),
            OsString::from("--"),
            OsString::from("-данные.md"),
        ])
        .unwrap();
        launch.resolve_cli_path().unwrap();
        assert_eq!(
            launch.path,
            Some(std::env::current_dir().unwrap().join("-данные.md"))
        );
        assert_eq!(launch.mode, Mode::View);
    }

    #[test]
    fn rejects_directory_with_a_retry_hint() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let error = Launch::from_path(directory).canonical_file().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("Выберите обычный файл"));
    }

    #[cfg(unix)]
    #[test]
    fn alias_keeps_the_requested_title_and_shares_the_file_identity() {
        use std::{fs, os::unix::fs::symlink};

        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let temp = std::env::temp_dir().join(format!("nimmd-launch-test-{}", std::process::id()));
        fs::create_dir(&temp).unwrap();
        let alias = temp.join("<название>.toml");
        symlink(&file, &alias).unwrap();
        let launch = Launch::from_path(alias.clone());
        let canonical = launch.canonical_file().unwrap();
        fs::remove_file(&alias).unwrap();
        fs::remove_dir(&temp).unwrap();

        assert_eq!(canonical, Some(file.canonicalize().unwrap()));
        assert_eq!(launch.path.as_deref(), Some(alias.as_path()));
        assert_eq!(launch.window_title(), "<название>.toml");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_native_path_with_a_retry_hint() {
        use std::os::unix::ffi::OsStringExt;

        let path = std::env::temp_dir().join(OsString::from_vec(vec![0xff]));
        let error = Launch::from_path(path).canonical_file().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("UTF-8"));
    }
}
