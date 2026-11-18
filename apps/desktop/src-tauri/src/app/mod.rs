#![forbid(unsafe_code)]

pub mod launch;

use std::io;

use tauri::Manager;

use launch::Launch;

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri извлекает State для команды по значению.
fn get_launch(state: tauri::State<'_, Launch>) -> Launch {
    state.inner().clone()
}

/// Запускает окно с режимом и путём из командной строки.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let launch = Launch::parse(std::env::args_os().skip(1))?;
    tauri::Builder::default()
        .manage(launch)
        .invoke_handler(tauri::generate_handler![get_launch])
        .setup(|app| {
            let window = app.get_webview_window("main").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Не создано главное окно")
            })?;
            window.set_title(&app.state::<Launch>().window_title())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
