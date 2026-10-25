//! Сборка встроенного дампа синтаксисов `assets/syntaxes.packdump` (рецепт `just syntaxes`):
//! стандартный набор syntect и грамматики из `assets/syntaxes/`.

use std::error::Error;
use std::path::Path;

use syntect::dumps::dump_to_uncompressed_file;
use syntect::parsing::SyntaxSet;

fn main() -> Result<(), Box<dyn Error>> {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_from_folder(assets.join("syntaxes"), true)?;
    let set = builder.build();
    if set.find_syntax_by_token("toml").is_none() {
        return Err("в наборе нет грамматики TOML".into());
    }
    // Без сжатия: ленивые синтаксисы в дампе уже сжаты
    dump_to_uncompressed_file(&set, assets.join("syntaxes.packdump"))?;
    println!("TOML: ok");
    Ok(())
}
