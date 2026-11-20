use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{Emitter, Manager, WebviewWindow};

use super::{
    RenderedEditor,
    save::{SaveOutcome, save_canonical},
    view::{DocumentFile, SourceFormat, revision},
    windows::Registry,
};

const EXTERNAL_CHANGE: &str = "nimmd-editor-external";

pub(super) fn watch(
    window: &WebviewWindow,
    file: &DocumentFile,
) -> Result<RecommendedWatcher, String> {
    let path = file.editor_path()?.to_owned();
    let parent = path
        .parent()
        .ok_or("У файла нет родительского каталога.")?
        .to_owned();
    let watched_dir = parent.clone();
    let app = window.app_handle().clone();
    let label = window.label().to_owned();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        if event.map_or(true, |event| {
            !event.kind.is_access()
                && (event.need_rescan()
                    || event.paths.is_empty()
                    || event.paths.contains(&path)
                    || event.paths.contains(&watched_dir))
        }) {
            let _ = app.emit_to(label.as_str(), EXTERNAL_CHANGE, ());
        }
    })
    .map_err(|error| error.to_string())?;
    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .map_err(|error| error.to_string())?;
    Ok(watcher)
}

fn file_for(window: &WebviewWindow) -> Result<DocumentFile, String> {
    let state = window.state::<Mutex<Registry>>();
    Registry::lock(state.inner())
        .file_for(window.label(), super::Mode::Edit)
        .ok_or_else(|| "У этого окна нет файла для редактирования.".to_owned())
}

#[tauri::command]
pub(super) async fn read_editor(window: WebviewWindow) -> Result<RenderedEditor, String> {
    let file = file_for(&window)?;
    let document = tauri::async_runtime::spawn_blocking(move || file.editor_latest())
        .await
        .map_err(|error| error.to_string())??;
    Ok(RenderedEditor {
        document,
        window_label: window.label().to_owned(),
        watch_warning: None,
    })
}

#[derive(serde::Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(super) enum SaveReply {
    Saved {
        revision: String,
        in_place: bool,
    },
    Conflict,
    Failed {
        message: String,
        recovery: Option<PathBuf>,
    },
}

#[tauri::command]
pub(super) async fn save_editor(
    window: WebviewWindow,
    source: String,
    revision: String,
    format: Option<SourceFormat>,
) -> Result<SaveReply, String> {
    let file = file_for(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = file.editor_path()?;
        Ok(save_file(path, &source, &revision, format))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn save_file(path: &Path, source: &str, expected: &str, format: Option<SourceFormat>) -> SaveReply {
    match save_canonical(path, source, format, |bytes| revision(bytes) == expected) {
        Ok(SaveOutcome::Saved { revision, in_place }) => {
            static WARNED: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
            let in_place = in_place
                && WARNED
                    .get_or_init(Mutex::default)
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(path.to_owned());
            SaveReply::Saved { revision, in_place }
        }
        Ok(SaveOutcome::Conflict) => SaveReply::Conflict,
        Ok(SaveOutcome::Recovered { path, error }) => SaveReply::Failed {
            message: error.to_string(),
            recovery: Some(path),
        },
        Err(error) => SaveReply::Failed {
            message: error.to_string(),
            recovery: None,
        },
    }
}

#[tauri::command]
pub(super) async fn recover_editor(
    window: WebviewWindow,
    source: String,
) -> Result<PathBuf, String> {
    let file = file_for(&window)?;
    let fallback = window
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string());
    tauri::async_runtime::spawn_blocking(move || {
        let path = file.editor_path()?;
        if let Some(parent) = path.parent()
            && let Ok(copy) = recovery_copy(parent, path, &source)
        {
            return Ok(copy);
        }
        let fallback = fallback?;
        fs::create_dir_all(&fallback).map_err(|error| error.to_string())?;
        recovery_copy(&fallback, path, &source).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn recovery_copy(directory: &Path, original: &Path, source: &str) -> io::Result<PathBuf> {
    let now = time::OffsetDateTime::now_utc();
    let (year, month, day) = now.to_calendar_date();
    let (hour, minute, second) = now.to_hms();
    let month = u8::from(month);
    let stamp = format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}");
    let name = original
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("document");
    // Учитываем ограничение длины имени файла на macOS.
    let (name, _) = nimmd_core::text::truncate_at_char_boundary(name, 180);
    let stem = format!("{name}.nimmd-recovery-{stamp}");
    for number in 0.. {
        let path = if number == 0 {
            directory.join(format!("{stem}.md"))
        } else {
            directory.join(format!("{stem}-{number}.md"))
        };
        let mut file = match OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        if let Err(error) = crate::platform::metadata::clear_inherited_acl(&file)
            .and_then(|()| file.write_all(source.as_bytes()))
            .and_then(|()| file.sync_all())
        {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        return Ok(path);
    }
    Err(io::Error::other("Не удалось выбрать имя резервной копии."))
}

pub(super) struct ExitIntent {
    restart: bool,
    waiting: String,
}

fn needs_editor_close(app: &tauri::AppHandle, label: &str) -> bool {
    Registry::lock(app.state::<Mutex<Registry>>().inner())
        .editor_sessions
        .contains(label)
}

fn ask_editor_close(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.set_focus();
        let _ = app.emit_to(label, "nimmd-editor-close", ());
    }
}

pub(super) fn handle_lifecycle(app: &tauri::AppHandle, event: &tauri::RunEvent) -> bool {
    use tauri::{RunEvent, WindowEvent};

    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if needs_editor_close(app, label) => {
            api.prevent_close();
            ask_editor_close(app, label);
        }
        RunEvent::ExitRequested {
            code: None | Some(0),
            api,
            ..
        } if begin_exit(app, false) => {
            api.prevent_exit();
        }
        _ => return false,
    }
    true
}

pub(super) fn begin_exit(app: &tauri::AppHandle, restart: bool) -> bool {
    let next = {
        let managed = app.state::<Mutex<Registry>>();
        let mut registry = Registry::lock(managed.inner());
        if registry.pending_exit.is_some() {
            return true;
        }
        let Some(label) = registry.editor_sessions.first().cloned() else {
            return false;
        };
        registry.pending_exit = Some(ExitIntent {
            restart,
            waiting: label.clone(),
        });
        label
    };
    ask_editor_close(app, &next);
    true
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri передаёт окно по значению.
pub(super) fn finish_editor_close(window: WebviewWindow, confirmed: bool) -> Result<(), String> {
    let app = window.app_handle();
    let restart = {
        let managed = app.state::<Mutex<Registry>>();
        let mut registry = Registry::lock(managed.inner());
        if registry
            .file_for(window.label(), super::Mode::Edit)
            .is_none()
        {
            return Err("У этого окна нет файла для редактирования.".to_owned());
        }
        let queued = registry
            .pending_exit
            .as_ref()
            .is_some_and(|intent| intent.waiting == window.label());
        if !confirmed {
            if queued {
                registry.pending_exit = None;
            }
            return Ok(());
        }
        let pending = queued.then(|| registry.pending_exit.take()).flatten();
        if pending.is_some() {
            registry.editor_sessions.remove(window.label());
        }
        pending.map(|intent| intent.restart)
    };

    if let Some(restart) = restart {
        if begin_exit(app, restart) {
            if let Err(error) = window.destroy() {
                // Уже подтверждённое и сохранённое окно также закроется при выходе.
                eprintln!("Не удалось закрыть окно NimMD: {error}");
            }
        } else if restart {
            app.request_restart();
        } else {
            app.exit(0);
        }
        return Ok(());
    }

    window.destroy().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use super::{SaveReply, recovery_copy, revision, save_file};

    #[test]
    fn rejects_external_edit_without_relying_on_a_watcher_event() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().canonicalize().unwrap().join("note.md");
        fs::write(&target, "read").unwrap();
        let first = save_file(&target, "first", &revision(b"read"), None);
        let SaveReply::Saved { revision, .. } = first else {
            panic!("первое сохранение должно завершиться успешно");
        };
        fs::write(&target, "external").unwrap();
        assert!(matches!(
            save_file(&target, "second", &revision, None),
            SaveReply::Conflict
        ));
        assert_eq!(fs::read_to_string(target).unwrap(), "external");
    }

    #[test]
    fn recovery_does_not_inherit_public_acl_entries() {
        let dir = tempfile::tempdir().unwrap();
        let inherited = std::process::Command::new("/bin/chmod")
            .args(["+a", "everyone allow read,file_inherit"])
            .arg(dir.path())
            .status()
            .unwrap();
        assert!(inherited.success());
        let ordinary = dir.path().join("control.md");
        fs::write(&ordinary, "control").unwrap();
        let copy = recovery_copy(dir.path(), &dir.path().join("note.md"), "private").unwrap();
        let acl = |path| {
            let listing = std::process::Command::new("/bin/ls")
                .arg("-le")
                .arg(path)
                .output()
                .unwrap();
            String::from_utf8_lossy(&listing.stdout).contains("inherited allow read")
        };
        assert!(acl(ordinary));
        assert!(!acl(copy), "резервная копия должна остаться приватной");
    }

    #[test]
    fn recovery_never_replaces_an_existing_private_copy() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("note.md");
        let first = recovery_copy(dir.path(), &original, "first").unwrap();
        let second = recovery_copy(dir.path(), &original, "second").unwrap();
        for (path, text) in [(first, "first"), (second, "second")] {
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
