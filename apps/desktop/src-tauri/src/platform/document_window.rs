use std::{io, path::Path, ptr::NonNull};

use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::NSWindow;
use objc2_foundation::{NSString, NSURL};
use tauri::WebviewWindow;

pub(crate) fn set_document_edited(window: &WebviewWindow, edited: bool) -> tauri::Result<()> {
    with_window(window, |native| native.setDocumentEdited(edited))
}

pub(crate) fn set_represented_file(window: &WebviewWindow, path: &Path) -> tauri::Result<()> {
    let url = represented_url(path)?;
    with_window(window, |native| native.setRepresentedURL(Some(&url)))
}

fn with_window(window: &WebviewWindow, update: impl FnOnce(&NSWindow)) -> tauri::Result<()> {
    let _main_thread = MainThreadMarker::new().ok_or_else(|| {
        io::Error::other("Заголовок окна можно обновлять только в главном потоке.")
    })?;
    let native = NonNull::new(window.ns_window()?.cast::<NSWindow>())
        .ok_or(tauri::Error::InvalidWindowHandle)?;
    // SAFETY: Tauri возвращает указатель на NSWindow, действительный в главном потоке
    // в пределах этого вызова. Ссылку не сохраняем и не передаём в другой поток.
    update(unsafe { native.as_ref() });
    Ok(())
}

fn represented_url(path: &Path) -> io::Result<Retained<NSURL>> {
    let path = path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Путь к файлу не в UTF-8."))?;
    Ok(NSURL::fileURLWithPath_isDirectory(
        &NSString::from_str(path),
        false,
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn represents_local_path_with_spaces_and_percent_sign() {
        let path = "/tmp/Заметка 100%.md";
        let url = super::represented_url(Path::new(path)).unwrap();
        assert!(url.isFileURL());
        assert_eq!(url.path().unwrap().to_string(), path);
        assert!(
            url.absoluteString()
                .unwrap()
                .to_string()
                .ends_with("%20100%25.md")
        );
    }
}
