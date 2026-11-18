use std::{ffi::OsString, io, path::PathBuf};

use serde::Serialize;

/// Режим окна, выбранный при запуске.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Edit,
    View,
}

/// Параметры запуска; файл пока не читается.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Launch {
    pub mode: Mode,
    pub path: Option<PathBuf>,
}

impl Launch {
    /// Разбирает аргументы без имени исполняемого файла.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> io::Result<Self> {
        let mut launch = Self {
            mode: Mode::Edit,
            path: None,
        };
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

    pub fn window_title(&self) -> String {
        let name = self
            .path
            .as_deref()
            .and_then(std::path::Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("NimMD");
        let mode = match self.mode {
            Mode::Edit => "Редактирование",
            Mode::View => "Просмотр",
        };
        format!("{name} — {mode}")
    }
}
