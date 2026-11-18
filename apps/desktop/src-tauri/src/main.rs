fn main() -> std::process::ExitCode {
    match nimmd_desktop::app::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Не удалось запустить NimMD: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
