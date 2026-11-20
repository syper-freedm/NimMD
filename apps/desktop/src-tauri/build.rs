fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_launch",
            "get_view",
            "get_editor",
            "highlight",
            "read_editor",
            "save_editor",
            "recover_editor",
            "finish_editor_close",
            "set_document_edited",
            "open_external",
        ]),
    ))?;
    Ok(())
}
