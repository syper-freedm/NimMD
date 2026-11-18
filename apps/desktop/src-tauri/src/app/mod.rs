#![forbid(unsafe_code)]

#[cfg(target_os = "macos")]
mod cli;
pub mod launch;
mod windows;

use std::{cell::Cell, rc::Rc, sync::Mutex};

use tauri::{DragDropEvent, RunEvent, WebviewWindow, WindowEvent};

use launch::Launch;
use windows::{Registry, WindowLaunch};

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

/// Запускает приложение или передаёт команду работающему экземпляру.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut launch = Launch::parse(std::env::args_os().skip(1))?;
    launch.resolve_cli_path()?;
    #[cfg(target_os = "macos")]
    let Some(primary) = crate::platform::instance::claim_or_forward(&launch)? else {
        return Ok(());
    };

    let app = tauri::Builder::default()
        .manage(Mutex::new(Registry::default()))
        .invoke_handler(tauri::generate_handler![get_launch])
        .build(tauri::generate_context!())?;
    #[cfg(target_os = "macos")]
    let mut primary = Some(primary);
    #[cfg(target_os = "macos")]
    let mut cli = None;

    let mut startup = Some((launch, Vec::new()));
    let mut ready = false;
    let startup_error = Rc::new(Cell::new(None));
    let error_out = Rc::clone(&startup_error);
    app.run_return(move |app, event| match event {
        RunEvent::Ready => ready = true,
        RunEvent::MainEventsCleared if ready => {
            if let Some((launch, urls)) = startup.take() {
                // Finder может передать первый документ до события Ready.
                if launch.path.is_some() || urls.is_empty() {
                    if let Err(error) = windows::open(app, launch) {
                        error_out.set(Some(error));
                        app.exit(1);
                        return;
                    }
                }
                windows::open_batch(app, None, urls.iter().map(Launch::from_url));
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
