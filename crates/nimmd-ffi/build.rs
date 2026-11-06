//! Генерирует `include/nimmd.h` через cbindgen при сборке крейта.

use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml")).unwrap_or_default();
    let header = crate_dir.join("include/nimmd.h");
    // Каталог include/ нужен Swift (#50); перезапись при сборке держит заголовок в синхроне с ABI.
    if std::fs::create_dir_all(crate_dir.join("include")).is_err() {
        eprintln!("cbindgen: не удалось создать include/");
        process::exit(1);
    }
    let bindings = match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cbindgen: не удалось сгенерировать заголовок: {e}");
            process::exit(1);
        }
    };
    bindings.write_to_file(header);
}
