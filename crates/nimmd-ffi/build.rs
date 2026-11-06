//! Генерирует `include/nimmd.h` через cbindgen при сборке крейта.

use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml")).unwrap_or_default();
    let ok = std::fs::create_dir_all(crate_dir.join("include")).is_ok()
        && cbindgen::Builder::new()
            .with_crate(&crate_dir)
            .with_config(config)
            .generate()
            .map(|b| b.write_to_file(crate_dir.join("include/nimmd.h")))
            .is_ok();
    if !ok {
        eprintln!("cbindgen: не удалось записать include/nimmd.h");
        process::exit(1);
    }
}
