fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_launch",
            "get_view",
            "get_editor",
            "open_external",
        ]),
    ))?;
    Ok(())
}
