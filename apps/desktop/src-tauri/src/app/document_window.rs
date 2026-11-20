use std::sync::Mutex;

use tauri::{Manager, WebviewWindow};

use crate::platform;

use super::{Mode, windows::Registry};

#[tauri::command]
pub(super) async fn set_document_edited(window: WebviewWindow, edited: bool) -> Result<(), String> {
    {
        let state = window.state::<Mutex<Registry>>();
        if Registry::lock(state.inner())
            .file_for(window.label(), Mode::Edit)
            .is_none()
        {
            return Err("У этого окна нет редактируемого файла.".to_owned());
        }
    }

    let native_window = window.clone();
    let (result, mut completed) = tauri::async_runtime::channel(1);
    window
        .run_on_main_thread(move || {
            let _ = result.try_send(platform::document_window::set_document_edited(
                &native_window,
                edited,
            ));
        })
        .map_err(|error| error.to_string())?;
    completed
        .recv()
        .await
        .ok_or_else(|| "Окно закрылось до обновления заголовка.".to_owned())?
        .map_err(|error| error.to_string())
}
