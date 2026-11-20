use std::{
    collections::HashMap,
    io,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use serde::Serialize;
use tauri::{
    AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    webview::NewWindowResponse,
};

use super::{
    assets::DocumentAssets,
    launch::{Launch, Mode},
    view::DocumentFile,
};

#[derive(Clone, Serialize)]
pub(super) struct WindowLaunch {
    #[serde(flatten)]
    launch: Launch,
    errors: Vec<String>,
    #[serde(skip)]
    file: Option<DocumentFile>,
}

#[derive(Default)]
pub(super) struct Registry {
    next_label: u64,
    by_file: HashMap<WindowKey, String>,
    by_label: HashMap<String, WindowLaunch>,
    assets: HashMap<String, Arc<DocumentAssets>>,
    #[cfg(target_os = "macos")]
    pub(super) watchers: HashMap<String, notify::RecommendedWatcher>,
    #[cfg(target_os = "macos")]
    pub(super) editor_sessions: std::collections::BTreeSet<String>,
    #[cfg(target_os = "macos")]
    pub(super) pending_exit: Option<super::edit::ExitIntent>,
}

#[derive(Eq, Hash, PartialEq)]
enum WindowKey {
    Empty,
    File(PathBuf),
}

pub(super) struct PreparedLaunch {
    key: Option<WindowKey>,
    data: WindowLaunch,
}

impl PreparedLaunch {
    pub(super) fn new(launch: Launch) -> io::Result<Self> {
        let (key, file) = if let Some(path) = &launch.path {
            Launch::validate_file_path(path)?;
            let (canonical, file) = DocumentFile::prepare(path, || Launch::canonical_file(path))?;
            (canonical.map(WindowKey::File), Some(file))
        } else {
            (Some(WindowKey::Empty), None)
        };
        Ok(Self {
            key,
            data: WindowLaunch {
                launch,
                errors: Vec::new(),
                file,
            },
        })
    }

    pub(super) fn has_file(&self) -> bool {
        self.data.launch.path.is_some()
    }
}

impl Registry {
    pub(super) fn lock(state: &Mutex<Self>) -> MutexGuard<'_, Self> {
        state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn launch(&self, label: &str) -> Option<WindowLaunch> {
        self.by_label.get(label).cloned()
    }

    pub(super) fn file_for(&self, label: &str, mode: Mode) -> Option<DocumentFile> {
        let data = self.by_label.get(label)?;
        if data.launch.mode == mode {
            data.file.clone()
        } else {
            None
        }
    }

    pub(super) fn assets(&self, label: &str) -> Option<Arc<DocumentAssets>> {
        self.assets.get(label).cloned()
    }

    fn register(&mut self, file: Option<WindowKey>, data: WindowLaunch) -> String {
        let label = format!("nimmd-window-{}", self.next_label);
        self.next_label += 1;
        if let Some(file) = file {
            if let WindowKey::File(path) = &file
                && let Some(directory) = path.parent()
                && let Ok(assets) = DocumentAssets::new(directory)
            {
                self.assets.insert(label.clone(), Arc::new(assets));
            }
            self.by_file.insert(file, label.clone());
        }
        self.by_label.insert(label.clone(), data);
        label
    }

    pub(super) fn remove(&mut self, label: &str) {
        self.assets.remove(label);
        #[cfg(target_os = "macos")]
        {
            self.watchers.remove(label);
            self.editor_sessions.remove(label);
        }
        self.by_label.remove(label);
        self.by_file.retain(|_, current| current != label);
    }
}

fn registry(app: &AppHandle) -> MutexGuard<'_, Registry> {
    Registry::lock(app.state::<Mutex<Registry>>().inner())
}

/// Оконные операции выполняются последовательно в главном потоке.
pub(super) fn open(
    app: &AppHandle,
    PreparedLaunch { key, data }: PreparedLaunch,
) -> io::Result<()> {
    let state = app.state::<Mutex<Registry>>();
    if let Some(key) = &key
        && reuse(state.inner(), key, |label| {
            let Some(window) = app.get_webview_window(label) else {
                return Ok(false);
            };
            match activate(&window) {
                Ok(()) => Ok(true),
                // Окно может быть ещё в реестре Tauri после начала закрытия.
                Err(error) if window.is_visible().is_ok() => {
                    Err(window_error(&data.launch.window_title(), error))
                }
                Err(_) => Ok(false),
            }
        })?
    {
        return Ok(());
    }

    create(app, key, data)
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
    #[cfg(target_os = "macos")]
    let represented_file = data.launch.path.clone();
    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .ok_or_else(|| window_error(&title, "В конфигурации отсутствуют параметры окна."))?;
    config.title.clone_from(&title);
    let page = if data.launch.mode == Mode::View {
        const VIEW_PAGE: &str = "view.html";
        config.url = WebviewUrl::App(VIEW_PAGE.into());
        VIEW_PAGE
    } else {
        // Tauri загружает index.html по корневому адресу приложения.
        ""
    };
    let page = page_url(app, config.use_https_scheme, page)
        .map_err(|error| window_error(&title, error))?;
    config.label = registry(app).register(file, data);

    let result = (|| {
        let window = WebviewWindowBuilder::from_config(app, &config)?
            .on_navigation(move |url| same_page(url, &page))
            .on_new_window(|_, _| NewWindowResponse::Deny)
            .build()?;
        let show = || {
            #[cfg(target_os = "macos")]
            if let Some(path) = represented_file.as_deref() {
                crate::platform::document_window::set_represented_file(&window, path)?;
            }
            activate(&window)
        };
        if let Err(error) = show() {
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

fn page_url(app: &AppHandle, https: bool, page: &str) -> io::Result<Url> {
    if tauri::is_dev()
        && let Some(dev_url) = &app.config().build.dev_url
    {
        return dev_url.join(page).map_err(io::Error::other);
    }
    // Tauri использует HTTP-адрес для внутреннего протокола на Windows и Android.
    let base = if cfg!(any(windows, target_os = "android")) {
        if https {
            "https://tauri.localhost/"
        } else {
            "http://tauri.localhost/"
        }
    } else {
        "tauri://localhost/"
    };
    Url::parse(base)
        .and_then(|base| base.join(page))
        .map_err(io::Error::other)
}

fn same_page(url: &Url, page: &Url) -> bool {
    let mut url = url.clone();
    url.set_fragment(None);
    if url.path().is_empty() && page.path() == "/" {
        url.set_path("/");
    }
    &url == page
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
    let errors = opening_errors(requests, |launch| open(app, PreparedLaunch::new(launch)?));
    if let Some(label) = source
        && let Some(window) = app.get_webview_window(label)
    {
        store_errors(&window, &errors);
        if !errors.is_empty() {
            let _ = activate(&window);
        }
        return;
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
            file: None,
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

    #[test]
    fn startup_hands_off_the_view_file_before_waking_the_window_thread() {
        use std::{fs, sync::mpsc, thread, time::Duration};

        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.md");
        fs::write(&path, "# Исходник").unwrap();
        let (wake, awakened) = mpsc::channel();
        let pending = super::super::prepare_startup(
            Launch {
                mode: Mode::View,
                path: Some(path.clone()),
            },
            move || wake.send(thread::current().id()).unwrap(),
        )
        .unwrap();
        let worker = awakened.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_ne!(worker, thread::current().id());
        let PreparedLaunch { key, data } = pending.try_recv().unwrap().unwrap();
        fs::rename(&path, folder.path().join("исходник.md")).unwrap();
        fs::write(&path, "# Замена").unwrap();

        let mut registry = Registry::default();
        let label = registry.register(key, data);
        let error = registry
            .file_for(&label, Mode::View)
            .unwrap()
            .render()
            .unwrap_err();
        assert!(error.contains("по этому пути теперь находится другой файл"));
    }

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
                file: None,
            },
        );
        let second = registry.register(
            Some(WindowKey::File(PathBuf::from("/b/note.md"))),
            WindowLaunch {
                launch: Launch::from_path(PathBuf::from("/b/note.md")),
                errors: Vec::new(),
                file: None,
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
    fn view_file_belongs_only_to_the_requesting_viewer() {
        let folder = tempfile::tempdir().unwrap();
        let mut registry = Registry::default();
        let mut register = |name: &str, content, mode| {
            let path = folder.path().join(name);
            std::fs::write(&path, content).unwrap();
            let PreparedLaunch { key, data } = PreparedLaunch::new(Launch {
                mode,
                path: Some(path),
            })
            .unwrap();
            registry.register(key, data)
        };
        let first = register("1.md", "Первый документ", Mode::View);
        let second = register("2.md", "Второй документ", Mode::View);
        let editor = register("3.md", "Документ в редакторе", Mode::Edit);
        let empty = registry.register(
            Some(WindowKey::Empty),
            WindowLaunch {
                launch: Launch::default(),
                errors: Vec::new(),
                file: None,
            },
        );
        let first_body = registry
            .file_for(&first, Mode::View)
            .unwrap()
            .render()
            .unwrap();
        let second_body = registry
            .file_for(&second, Mode::View)
            .unwrap()
            .render()
            .unwrap();
        assert!(first_body.starts_with(r#"<main class="nimmd">"#));
        assert!(first_body.contains("Первый документ"));
        assert!(!first_body.contains("Второй документ"));
        assert!(second_body.contains("Второй документ"));
        assert!(!second_body.contains("Первый документ"));
        assert!(registry.file_for(&editor, Mode::View).is_none());
        assert!(registry.file_for(&first, Mode::Edit).is_none());
        let document = registry
            .file_for(&editor, Mode::Edit)
            .unwrap()
            .editor_document()
            .unwrap();
        assert_eq!(document.source.as_deref(), Some("Документ в редакторе"));
        assert!(registry.file_for(&empty, Mode::View).is_none());
        assert!(registry.file_for("unknown-window", Mode::View).is_none());
    }

    #[test]
    fn view_launch_rejects_missing_file_and_directory() {
        let folder = tempfile::tempdir().unwrap();
        for (path, expected) in [
            (folder.path().join("нет.md"), io::ErrorKind::NotFound),
            (folder.path().to_owned(), io::ErrorKind::InvalidInput),
        ] {
            let error = PreparedLaunch::new(Launch {
                mode: Mode::View,
                path: Some(path),
            })
            .err()
            .unwrap();
            assert_eq!(error.kind(), expected);
            assert!(error.to_string().contains("Не удалось открыть"));
        }
    }

    #[test]
    fn viewer_navigation_allows_only_its_page_and_anchors() {
        let page = Url::parse("tauri://localhost/view.html").unwrap();
        assert!(same_page(&page, &page));
        assert!(same_page(&page.join("#fn-1").unwrap(), &page));
        for url in [
            "tauri://localhost/index.html",
            "tauri://localhost/view.html?file=note.md",
            "https://example.com/",
        ] {
            assert!(!same_page(&Url::parse(url).unwrap(), &page));
        }
    }

    #[test]
    fn editor_navigation_allows_both_spellings_of_the_tauri_root() {
        let page = Url::parse("tauri://localhost/").unwrap();
        for url in [
            "tauri://localhost",
            "tauri://localhost/",
            "tauri://localhost#anchor",
        ] {
            assert!(same_page(&Url::parse(url).unwrap(), &page), "{url}");
        }
        for url in [
            "tauri://localhost/view.html",
            "tauri://elsewhere/",
            "file:///",
        ] {
            assert!(!same_page(&Url::parse(url).unwrap(), &page), "{url}");
        }
    }
    #[test]
    fn assets_are_scoped_to_the_open_document_and_window_lifetime() {
        let directory = tempfile::tempdir().unwrap();
        let mut registry = Registry::default();
        let register = |name: &str, contents: u8| {
            let folder = directory.path().join(name);
            std::fs::create_dir(&folder).unwrap();
            let path = folder.join("note.md");
            std::fs::write(&path, "").unwrap();
            for suffix in 1..=5 {
                std::fs::write(
                    folder.join(format!("ф #{suffix}.PNG")),
                    vec![contents; 5 * 1024 * 1024],
                )
                .unwrap();
            }
            PreparedLaunch::new(Launch::from_path(path)).unwrap()
        };
        let PreparedLaunch { key, data } = register("first", 1);
        let first = registry.register(key, data);
        let PreparedLaunch { key, data } = register("second", 2);
        let second = registry.register(key, data);
        for suffix in 1..=4 {
            let resource = format!("%D1%84%20%23{suffix}.PNG");
            let image = registry.assets(&first).unwrap().read(&resource).unwrap();
            assert_eq!(image.content_type, "image/png");
            assert_eq!(image.bytes.first(), Some(&1));
        }
        let current = registry.assets(&first).unwrap();
        assert!(current.read("%D1%84%20%231.PNG").is_ok());
        assert!(current.read("%D1%84%20%235.PNG").is_err());
        let other = registry.assets(&second).unwrap();
        assert_eq!(
            other.read("%D1%84%20%235.PNG").unwrap().bytes.first(),
            Some(&2)
        );
        let original_path = directory.path().join("first/note.md");
        registry.remove(&first);
        assert!(registry.assets(&first).is_none());
        let PreparedLaunch { key, data } =
            PreparedLaunch::new(Launch::from_path(original_path)).unwrap();
        let reopened = registry.register(key, data);
        assert!(
            registry
                .assets(&reopened)
                .unwrap()
                .read("%D1%84%20%235.PNG")
                .is_ok()
        );
        assert!(registry.assets("unknown").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn reuses_existing_windows_when_view_read_access_is_denied() {
        use std::{
            fs,
            os::unix::fs::{PermissionsExt, symlink},
        };

        if rustix::process::geteuid().is_root() {
            return;
        }
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("заметка.md");
        let alias = folder.path().join("ссылка.md");
        fs::write(&path, "# Исходник").unwrap();
        symlink(&path, &alias).unwrap();

        let state = Mutex::new(Registry::default());
        let PreparedLaunch { key, data } =
            PreparedLaunch::new(Launch::from_path(path.clone())).unwrap();
        let edit_label = Registry::lock(&state).register(key, data);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        let request = || {
            PreparedLaunch::new(Launch {
                mode: Mode::View,
                path: Some(alias.clone()),
            })
            .unwrap()
        };
        let PreparedLaunch { key, data } = request();
        let key = key.expect("Канонический путь недоступного файла служит ключом окна");
        assert!(
            reuse(&state, &key, |label| {
                assert_eq!(label, edit_label);
                Ok(true)
            })
            .unwrap()
        );
        assert_eq!(
            Registry::lock(&state)
                .launch(&edit_label)
                .unwrap()
                .launch
                .mode,
            Mode::Edit
        );

        Registry::lock(&state).remove(&edit_label);
        let view_label = Registry::lock(&state).register(Some(key), data);
        let repeated_key = request().key.unwrap();
        assert!(
            reuse(&state, &repeated_key, |label| {
                assert_eq!(label, view_label);
                Ok(true)
            })
            .unwrap()
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
                file: None,
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
                file: None,
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
            PreparedLaunch::new(launch.clone())?;
            opened.push(launch);
            Ok(())
        });
        assert_eq!(opened, [Launch::from_path(path)]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("bad.md"));
        assert!(errors[0].contains("локальный файл"));
    }
}
