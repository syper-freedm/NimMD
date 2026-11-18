use std::{
    collections::HashMap,
    io,
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use super::launch::Launch;

#[derive(Clone, Serialize)]
pub(super) struct WindowLaunch {
    #[serde(flatten)]
    launch: Launch,
    errors: Vec<String>,
}

#[derive(Default)]
pub(super) struct Registry {
    next_label: u64,
    by_file: HashMap<WindowKey, String>,
    by_label: HashMap<String, WindowLaunch>,
}

#[derive(Eq, Hash, PartialEq)]
enum WindowKey {
    Empty,
    File(PathBuf),
}

impl Registry {
    pub(super) fn lock(state: &Mutex<Self>) -> MutexGuard<'_, Self> {
        state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn launch(&self, label: &str) -> Option<WindowLaunch> {
        self.by_label.get(label).cloned()
    }

    fn register(&mut self, file: Option<WindowKey>, data: WindowLaunch) -> String {
        let label = format!("nimmd-window-{}", self.next_label);
        self.next_label += 1;
        if let Some(file) = file {
            self.by_file.insert(file, label.clone());
        }
        self.by_label.insert(label.clone(), data);
        label
    }

    pub(super) fn remove(&mut self, label: &str) {
        self.by_label.remove(label);
        self.by_file.retain(|_, current| current != label);
    }
}

fn registry(app: &AppHandle) -> MutexGuard<'_, Registry> {
    Registry::lock(app.state::<Mutex<Registry>>().inner())
}

/// Оконные операции выполняются последовательно в главном потоке.
pub(super) fn open(app: &AppHandle, launch: Launch) -> io::Result<()> {
    let file = launch
        .canonical_file()?
        .map_or(WindowKey::Empty, WindowKey::File);
    let state = app.state::<Mutex<Registry>>();
    let reused = reuse(state.inner(), &file, |label| {
        let Some(window) = app.get_webview_window(label) else {
            return Ok(false);
        };
        match activate(&window) {
            Ok(()) => Ok(true),
            // Окно может быть ещё в реестре Tauri после начала закрытия.
            Err(error) if window.is_visible().is_ok() => {
                Err(window_error(&launch.window_title(), error))
            }
            Err(_) => Ok(false),
        }
    })?;
    if reused {
        return Ok(());
    }

    create(
        app,
        Some(file),
        WindowLaunch {
            launch,
            errors: Vec::new(),
        },
    )
}

fn reuse(
    state: &Mutex<Registry>,
    file: &WindowKey,
    activate: impl FnOnce(&str) -> io::Result<bool>,
) -> io::Result<bool> {
    let Some(label) = Registry::lock(state).by_file.get(file).cloned() else {
        return Ok(false);
    };
    if activate(&label)? {
        return Ok(true);
    }
    Registry::lock(state).remove(&label);
    Ok(false)
}

fn create(app: &AppHandle, file: Option<WindowKey>, data: WindowLaunch) -> io::Result<()> {
    let title = data.launch.window_title();
    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .ok_or_else(|| window_error(&title, "В конфигурации отсутствуют параметры окна."))?;
    config.title.clone_from(&title);
    config.label = registry(app).register(file, data);

    let result = (|| {
        let window = WebviewWindowBuilder::from_config(app, &config)?.build()?;
        if let Err(error) = activate(&window) {
            let _ = window.destroy();
            return Err(error);
        }
        Ok(())
    })();
    if let Err(error) = result {
        registry(app).remove(&config.label);
        return Err(window_error(&title, error));
    }
    Ok(())
}

fn activate(window: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    window.app_handle().show()?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    // Сеттеры Tauri могут вернуть успех, пока нативное окно уже закрывается.
    window.is_visible()?;
    Ok(())
}

fn window_error(title: &str, error: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!(
        "Не удалось показать окно «{title}». {error} Повторите открытие."
    ))
}

pub(super) fn open_batch(
    app: &AppHandle,
    source: Option<&str>,
    requests: impl Iterator<Item = io::Result<Launch>>,
) {
    let errors = opening_errors(requests, |launch| open(app, launch));
    if let Some(label) = source {
        if let Some(window) = app.get_webview_window(label) {
            store_errors(&window, &errors);
            if !errors.is_empty() {
                let _ = activate(&window);
            }
            return;
        }
    }
    if errors.is_empty() {
        return;
    }

    let open_windows = app.webview_windows();
    let focused = open_windows
        .values()
        .find(|window| window.is_focused().unwrap_or(false))
        .cloned();
    let candidates = focused.into_iter().chain(open_windows.into_values());
    for window in candidates {
        if activate(&window).is_ok() {
            store_errors(&window, &errors);
            return;
        }
    }
    if let Err(error) = create(
        app,
        None,
        WindowLaunch {
            launch: Launch::default(),
            errors: errors.clone(),
        },
    ) {
        eprintln!("{}\n{error}", errors.join("\n"));
    }
}

fn store_errors(window: &WebviewWindow, errors: &[String]) {
    let changed = {
        let mut windows = registry(window.app_handle());
        if let Some(data) = windows.by_label.get_mut(window.label()) {
            let changed = data.errors != errors;
            if changed {
                data.errors = errors.to_vec();
            }
            changed
        } else {
            false
        }
    };
    if changed {
        // Данные хранятся в Rust, даже если скрипт окна ещё не загрузился.
        let _ = window.eval(
            "document.getElementById('app')?.dispatchEvent(new Event('nimmd-launch-changed'))",
        );
    }
}

fn opening_errors(
    requests: impl Iterator<Item = io::Result<Launch>>,
    mut open: impl FnMut(Launch) -> io::Result<()>,
) -> Vec<String> {
    requests
        .filter_map(|request| request.and_then(&mut open).err())
        .map(|error| error.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::launch::Mode;

    #[test]
    fn launch_data_belongs_to_the_requesting_window() {
        let mut registry = Registry::default();
        let first = registry.register(
            Some(WindowKey::File(PathBuf::from("/a/note.md"))),
            WindowLaunch {
                launch: Launch {
                    mode: Mode::View,
                    path: Some(PathBuf::from("/a/<note>.md")),
                },
                errors: vec!["Выберите файл.".into()],
            },
        );
        let second = registry.register(
            Some(WindowKey::File(PathBuf::from("/b/note.md"))),
            WindowLaunch {
                launch: Launch::from_path(PathBuf::from("/b/note.md")),
                errors: Vec::new(),
            },
        );
        assert_eq!(
            serde_json::to_value(registry.launch(&first).unwrap()).unwrap(),
            serde_json::json!({"mode": "view", "path": "/a/<note>.md", "errors": ["Выберите файл."]})
        );
        assert_eq!(
            serde_json::to_value(registry.launch(&second).unwrap()).unwrap(),
            serde_json::json!({"mode": "edit", "path": "/b/note.md", "errors": []})
        );
    }

    #[test]
    fn closing_between_lookup_and_activation_allows_reopen() {
        let state = Mutex::new(Registry::default());
        let file = || WindowKey::File(PathBuf::from("/note.md"));
        let first = Registry::lock(&state).register(
            Some(file()),
            WindowLaunch {
                launch: Launch::from_path(PathBuf::from("/note.md")),
                errors: Vec::new(),
            },
        );
        assert!(
            !reuse(&state, &file(), |label| {
                assert_eq!(label, first);
                Ok(false) // Нативное окно закрылось в ходе активации.
            })
            .unwrap()
        );
        let second = Registry::lock(&state).register(
            Some(file()),
            WindowLaunch {
                launch: Launch {
                    mode: Mode::View,
                    path: Some(PathBuf::from("/note.md")),
                },
                errors: Vec::new(),
            },
        );
        Registry::lock(&state).remove(&first); // Запоздавшее событие Destroyed.
        assert!(
            reuse(&state, &file(), |label| {
                assert_eq!(label, second);
                Ok(true)
            })
            .unwrap()
        );
        let registry = Registry::lock(&state);
        assert!(registry.launch(&first).is_none());
        assert_eq!(registry.launch(&second).unwrap().launch.mode, Mode::View);
    }

    #[test]
    fn batch_opens_the_local_file_after_a_bad_url() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let urls = [
            tauri::Url::parse("file://example.invalid/bad.md").unwrap(),
            tauri::Url::from_file_path(&path).unwrap(),
        ];
        let mut opened = Vec::new();
        let errors = opening_errors(urls.iter().map(Launch::from_url), |launch| {
            launch.canonical_file()?;
            opened.push(launch);
            Ok(())
        });
        assert_eq!(opened, [Launch::from_path(path)]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("bad.md"));
        assert!(errors[0].contains("локальный файл"));
    }
}
