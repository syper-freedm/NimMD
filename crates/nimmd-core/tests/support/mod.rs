//! Общие вспомогательные функции интеграционных тестов.

use std::path::{Path, PathBuf};
use std::{fs, io};

/// Детерминированный генератор xorshift64 для воспроизводимых прогонов.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Markdown-фикстуры внутри каталогов групп; файлы рядом с группами пропускаются.
#[allow(dead_code)] // Часть тестовых бинарников использует только Rng.
pub fn markdown_fixtures(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for group in fs::read_dir(root)? {
        let group = group?.path();
        if group.is_dir() {
            for entry in fs::read_dir(group)? {
                let path = entry?.path();
                if path.extension().is_some_and(|ext| ext == "md") {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}
