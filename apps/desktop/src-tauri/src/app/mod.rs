#![forbid(unsafe_code)]

mod assets;
#[cfg(target_os = "macos")]
mod cli;
pub mod launch;
mod view;
mod windows;

use std::{
    cell::Cell,
    io,
    rc::Rc,
    sync::{
        Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
};

use tauri::{AppHandle, DragDropEvent, RunEvent, Url, WebviewWindow, WindowEvent};
use tauri_plugin_opener::OpenerExt;

use launch::Launch;
use windows::{PreparedLaunch, Registry, WindowLaunch};

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri извлекает окно и State для команды по значению.
fn get_launch(
    window: WebviewWindow,
    state: tauri::State<'_, Mutex<Registry>>,
) -> Result<WindowLaunch, String> {
    Registry::lock(state.inner())
        .launch(window.label())
        .ok_or_else(|| "Параметры окна недоступны".into())
}

#[tauri::command]
async fn get_view(
    window: WebviewWindow,
    state: tauri::State<'_, Mutex<Registry>>,
) -> Result<RenderedView, String> {
    let file = Registry::lock(state.inner())
        .view_file(window.label())
        .ok_or_else(|| "У этого окна нет файла для просмотра.".to_owned())?;
    let body = tauri::async_runtime::spawn_blocking(move || file.render())
        .await
        .map_err(|_| {
            "Не удалось подготовить просмотр. Закройте окно и откройте файл снова.".to_owned()
        })??;
    Ok(RenderedView {
        body,
        window_label: window.label().to_owned(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderedView {
    body: String,
    window_label: String,
}

fn external_url(input: &str) -> Option<Url> {
    let url = Url::parse(input).ok()?;
    match url.scheme() {
        "http" | "https" if url.has_host() => Some(url),
        "mailto"
            if !url.path().is_empty() || url.query().is_some_and(|query| !query.is_empty()) =>
        {
            Some(url)
        }
        _ => None,
    }
}

#[tauri::command]
async fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    let url = external_url(&url).ok_or_else(|| "Схема ссылки не поддерживается.".to_owned())?;
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|_| "Не удалось открыть ссылку в системном приложении.".to_owned())
}

fn prepare_startup(
    launch: Launch,
    wake: impl FnOnce() + Send + 'static,
) -> io::Result<Receiver<io::Result<PreparedLaunch>>> {
    let (prepared, pending) = mpsc::channel();
    thread::Builder::new()
        .name("nimmd-startup".into())
        .spawn(move || {
            if prepared.send(PreparedLaunch::new(launch)).is_ok() {
                wake();
            }
        })?;
    Ok(pending)
}

fn build_app() -> tauri::Result<tauri::App> {
    tauri::Builder::default()
        .manage(Mutex::new(Registry::default()))
        // Встроенный скрипт Opener обрабатывает больше схем, чем разрешено NimMD.
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .register_asynchronous_uri_scheme_protocol(assets::SCHEME, |context, request, responder| {
            assets::serve(&context, &request, responder);
        })
        .invoke_handler(tauri::generate_handler![
            get_launch,
            get_view,
            open_external
        ])
        .build(tauri::generate_context!())
}

/// Запускает приложение или передаёт команду работающему экземпляру.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut launch = Launch::parse(std::env::args_os().skip(1))?;
    launch.resolve_cli_path()?;
    #[cfg(target_os = "macos")]
    let Some(primary) = crate::platform::instance::claim_or_forward(&launch)? else {
        return Ok(());
    };

    let app = build_app()?;
    #[cfg(target_os = "macos")]
    let mut primary = Some(primary);
    #[cfg(target_os = "macos")]
    let mut cli = None;

    let startup_app = app.handle().clone();
    let pending = prepare_startup(launch, move || {
        let _ = startup_app.run_on_main_thread(|| {});
    })?;
    let mut startup = Some((pending, Vec::new()));
    let mut ready = false;
    let startup_error = Rc::new(Cell::new(None));
    let error_out = Rc::clone(&startup_error);
    app.run_return(move |app, event| match event {
        RunEvent::Ready => ready = true,
        RunEvent::MainEventsCleared if ready => {
            if let Some((pending, urls)) = &startup {
                let prepared = match pending.try_recv() {
                    Ok(result) => result,
                    Err(TryRecvError::Empty) => return,
                    Err(error) => Err(io::Error::other(error)),
                };
                let result = prepared.and_then(|launch| {
                    // Finder может передать первый документ до готовности запуска.
                    if launch.has_file() || urls.is_empty() {
                        windows::open(app, launch)?;
                    }
                    windows::open_batch(app, None, urls.iter().map(Launch::from_url));
                    Ok(())
                });
                startup = None;
                if let Err(error) = result {
                    error_out.set(Some(error));
                    app.exit(1);
                    return;
                }
                #[cfg(target_os = "macos")]
                if let Some(primary) = primary.take() {
                    // При ошибке первого открытия клиент не должен получить сигнал готовности.
                    match cli::Requests::start(primary, app) {
                        Ok(requests) => cli = Some(requests),
                        Err(error) => {
                            error_out.set(Some(error));
                            app.exit(1);
                            return;
                        }
                    }
                }
            }
            #[cfg(target_os = "macos")]
            if let Some(cli) = &mut cli {
                cli.drain(app);
            }
        }
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
        RunEvent::Opened { urls } => {
            if let Some((_, early_urls)) = &mut startup {
                early_urls.extend(urls);
            } else {
                windows::open_batch(app, None, urls.iter().map(Launch::from_url));
            }
        }
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }),
            ..
        } => windows::open_batch(
            app,
            Some(&label),
            paths.into_iter().map(|path| Ok(Launch::from_path(path))),
        ),
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::Destroyed,
            ..
        } => {
            use tauri::Manager;
            Registry::lock(app.state::<Mutex<Registry>>().inner()).remove(&label);
        }
        #[cfg(target_os = "macos")]
        RunEvent::ExitRequested {
            code: None, api, ..
        } if cli
            .as_mut()
            .is_some_and(cli::Requests::defer_automatic_exit) =>
        {
            api.prevent_exit();
        }
        _ => {}
    });
    startup_error
        .take()
        .map_or(Ok(()), |error| Err(error.into()))
}

#[cfg(test)]
mod tests {
    use super::external_url;

    #[test]
    fn only_external_web_and_mail_links_are_opened() {
        for url in [
            "http://example.com/",
            "HTTPS://example.com/",
            "mailto:a@example.com",
            "mailto:?subject=NimMD",
        ] {
            assert!(external_url(url).is_some(), "{url}");
        }
        for url in [
            "file:///tmp/note.md",
            "tel:+123",
            "//example.com",
            "nimmd-asset://localhost/path",
            "mailto:",
        ] {
            assert!(external_url(url).is_none(), "{url}");
        }
    }
}
