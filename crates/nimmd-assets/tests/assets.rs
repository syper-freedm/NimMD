use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nimmd_assets::{AssetBudget, AssetError, MAX_ASSET_BYTES, MAX_BUDGET_BYTES, read_asset};

/// Чистый каталог теста в `target/tmp`: у каждого теста свой, тесты идут параллельно.
fn fresh_dir(name: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("nimmd-assets")
        .join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

fn filled(len: u64, byte: u8) -> Vec<u8> {
    vec![byte; usize::try_from(len).unwrap_or(0)]
}

/// Каталог с документом `doc.md`; возвращает путь документа.
fn doc_in(name: &str) -> io::Result<PathBuf> {
    let doc = fresh_dir(name)?.join("doc.md");
    write(&doc, b"# doc\n")?;
    Ok(doc)
}

const NO_DOC: &str = "/nonexistent-nimmd/doc.md";

#[test]
fn reads_images_next_to_document_and_counts_each_file_once() {
    let dir = fresh_dir("sc001").unwrap().join("docs");
    let doc = dir.join("note.md");
    write(&doc, b"# note\n").unwrap();
    let a = b"0123456789".to_vec();
    let jpeg = b"jpeg-bytes".repeat(3);
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec();
    write(&dir.join("img").join("a.png"), &a).unwrap();
    write(&dir.join("b.JPEG"), &jpeg).unwrap();
    write(&dir.join("c d.svg"), &svg).unwrap();

    let mut budget = AssetBudget::new();
    for (href, expected) in [
        ("img/a.png", &a),
        ("./img/../img/a.png", &a),
        ("img\\a.png", &a),
        ("b.JPEG", &jpeg),
        ("c%20d.svg", &svg),
    ] {
        assert_eq!(
            read_asset(&doc, href, &mut budget).as_ref(),
            Ok(expected),
            "{href}"
        );
    }
    assert_eq!(budget.used(), (a.len() + jpeg.len() + svg.len()) as u64);
}

#[test]
fn rejects_before_touching_filesystem() {
    let doc = Path::new(NO_DOC);
    let mut budget = AssetBudget::new();
    let cases = [
        ("../x.png", AssetError::Escape),
        ("a/../../x.png", AssetError::Escape),
        ("%2e%2e/x.png", AssetError::Escape),
        ("%2E%2E%2Fx.png", AssetError::Escape),
        ("..\\x.png", AssetError::Escape),
        ("/etc/hosts", AssetError::Absolute),
        ("\\x.png", AssetError::Absolute),
        ("~/x.png", AssetError::Home),
        ("file:///x.png", AssetError::Scheme),
        ("http://h/x.png", AssetError::Scheme),
        ("C:/x.png", AssetError::Scheme),
        ("%zz.png", AssetError::BadEncoding),
        ("%ff.png", AssetError::BadEncoding),
        ("%00.png", AssetError::BadEncoding),
        ("", AssetError::BadEncoding),
    ];
    for (href, expected) in cases {
        assert_eq!(
            read_asset(doc, href, &mut budget),
            Err(expected),
            "{href:?}"
        );
    }
    assert_eq!(budget.used(), 0);
}

#[test]
fn rejection_edge_cases() {
    let doc = Path::new(NO_DOC);
    let mut budget = AssetBudget::new();
    let cases = [
        // `%` без двух цифр
        ("%", AssetError::BadEncoding),
        ("x.png%", AssetError::BadEncoding),
        ("x.png%2", AssetError::BadEncoding),
        ("%2g.png", AssetError::BadEncoding),
        // пустой результат и NUL в середине
        ("a\0.png", AssetError::BadEncoding),
        // схема: регистр, `+.-`, закодированное двоеточие
        ("HTTPS://h/x.png", AssetError::Scheme),
        ("data:image/png;base64,AA", AssetError::Scheme),
        ("a+b.c-d:x.png", AssetError::Scheme),
        ("c%3Ax.png", AssetError::Scheme),
        // абсолютный и домашний — после декодирования
        ("%2Fetc%2Fhosts", AssetError::Absolute),
        ("%5Cx.png", AssetError::Absolute),
        ("%7E/x.png", AssetError::Home),
        ("~user/x.png", AssetError::Home),
        // `..` снимает компонент, выход за пределы — где угодно в пути
        ("a/b/../../../x.png", AssetError::Escape),
        ("./..", AssetError::Escape),
        ("a\\..\\..\\x.png", AssetError::Escape),
        ("a/%2E%2E/..%5Cx.png", AssetError::Escape),
    ];
    for (href, expected) in cases {
        assert_eq!(
            read_asset(doc, href, &mut budget),
            Err(expected),
            "{href:?}"
        );
    }
    // Не схема и не выход: доходит до ФС, а документа нет
    for href in [
        "1a:x.png",
        ":x.png",
        "a/../x.png",
        "a/./b/../../x.png",
        "x~.png",
        "%252e%252e/x.png",
        "..x.png",
    ] {
        assert_eq!(
            read_asset(doc, href, &mut budget),
            Err(AssetError::Io),
            "{href:?}"
        );
    }
    assert_eq!(budget.used(), 0);
}

#[test]
fn decodes_once_not_twice() {
    let doc = doc_in("decode-once").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("%2e.png"), b"literal").unwrap();
    write(&dir.join("a b.png"), b"space").unwrap();
    write(&dir.join("я.png"), b"cyrillic").unwrap();
    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&doc, "%252e.png", &mut budget),
        Ok(b"literal".to_vec())
    );
    assert_eq!(
        read_asset(&doc, "a%20b.png", &mut budget),
        Ok(b"space".to_vec())
    );
    assert_eq!(
        read_asset(&doc, "%D1%8F.png", &mut budget),
        Ok(b"cyrillic".to_vec())
    );
    assert_eq!(
        read_asset(&doc, "%d1%8f.png", &mut budget),
        Ok(b"cyrillic".to_vec())
    );
    assert_eq!(budget.used(), 7 + 5 + 8);
}

/// Простой линейный конгруэнтный генератор: фиксированное зерно, воспроизводимый прогон.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).map_or(0, |x| x % n)
    }
}

#[test]
fn percent_encoding_every_byte_does_not_change_verdict() {
    // Эталон — та же ссылка без кодирования: однократный decode `%XX` каждого байта
    // должен давать тот же отказ шагов 1–3 (без `%` и NUL в алфавите).
    let alphabet: Vec<char> = "ab.C/\\~:+-1я ".chars().collect();
    let doc = Path::new(NO_DOC);
    let mut rng = Lcg(0x6e_696d_6d64);
    for _ in 0..2000 {
        let len = rng.below(8);
        let raw: String = (0..len)
            .map(|_| alphabet[rng.below(alphabet.len())])
            .collect();
        let encoded: String = raw
            .bytes()
            .map(|b| {
                if rng.below(2) == 0 {
                    format!("%{b:02x}")
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect();
        let mut b1 = AssetBudget::new();
        let mut b2 = AssetBudget::new();
        assert_eq!(
            read_asset(doc, &encoded, &mut b1),
            read_asset(doc, &raw, &mut b2),
            "raw {raw:?}, encoded {encoded:?}"
        );
        assert!(read_asset(doc, &raw, &mut b2).is_err());
        assert_eq!(b1.used() + b2.used(), 0);
    }
}

#[cfg(unix)]
#[test]
fn symlink_inside_directory_leading_out_escapes() {
    use std::os::unix::fs::symlink;
    let root = fresh_dir("sc003").unwrap();
    let base = root.join("base");
    let doc = base.join("doc.md");
    write(&doc, b"# doc\n").unwrap();
    write(&root.join("outside").join("secret.png"), b"secret").unwrap();
    symlink("../outside/secret.png", base.join("link.png")).unwrap();
    symlink("../outside", base.join("out")).unwrap();

    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&doc, "link.png", &mut budget),
        Err(AssetError::Escape)
    );
    assert_eq!(
        read_asset(&doc, "out/secret.png", &mut budget),
        Err(AssetError::Escape)
    );
    assert_eq!(budget.used(), 0);
}

#[cfg(unix)]
#[test]
fn document_symlink_resolves_base_to_target_directory() {
    use std::os::unix::fs::symlink;
    let root = fresh_dir("sc004").unwrap();
    let doc = root.join("base").join("doc.md");
    write(&doc, b"# doc\n").unwrap();
    write(&root.join("base").join("a.png"), b"base-a").unwrap();
    write(&root.join("other").join("a.png"), b"other-a").unwrap();
    let alias = root.join("other").join("alias.md");
    symlink(&doc, &alias).unwrap();

    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&alias, "a.png", &mut budget),
        Ok(b"base-a".to_vec())
    );
    // Тот же канонический файл через документ — бюджет не расходуется повторно
    assert_eq!(
        read_asset(&doc, "a.png", &mut budget),
        Ok(b"base-a".to_vec())
    );
    assert_eq!(budget.used(), 6);
}

#[cfg(unix)]
#[test]
fn symlink_to_image_inside_directory_is_counted_once() {
    use std::os::unix::fs::symlink;
    let doc = doc_in("symlink-inside").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("img").join("a.png"), b"abc").unwrap();
    symlink("img/a.png", dir.join("alias.png")).unwrap();

    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&doc, "alias.png", &mut budget),
        Ok(b"abc".to_vec())
    );
    assert_eq!(
        read_asset(&doc, "img/a.png", &mut budget),
        Ok(b"abc".to_vec())
    );
    assert_eq!(budget.used(), 3);
}

#[test]
fn only_regular_image_files_are_read() {
    let doc = doc_in("sc005").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("notes.txt"), b"text").unwrap();
    write(&dir.join("x.png.txt"), b"text").unwrap();
    write(&dir.join("noext"), b"text").unwrap();
    write(&dir.join("png"), b"text").unwrap();
    write(&dir.join(".png"), b"text").unwrap();
    fs::create_dir(dir.join("dir.png")).unwrap();
    write(&dir.join("UP.PnG"), b"upper").unwrap();

    let mut budget = AssetBudget::new();
    for href in [
        "notes.txt",
        "x.png.txt",
        "noext",
        "png",
        ".png",
        "dir.png",
        "doc.md",
    ] {
        assert_eq!(
            read_asset(&doc, href, &mut budget),
            Err(AssetError::NotImage),
            "{href}"
        );
    }
    assert_eq!(
        read_asset(&doc, "missing.png", &mut budget),
        Err(AssetError::Io)
    );
    assert_eq!(budget.used(), 0);
    assert_eq!(
        read_asset(&doc, "UP.PnG", &mut budget),
        Ok(b"upper".to_vec())
    );
    for ext in ["png", "jpg", "jpeg", "gif", "webp", "svg"] {
        let name = format!("f.{ext}");
        write(&dir.join(&name), ext.as_bytes()).unwrap();
        assert_eq!(
            read_asset(&doc, &name, &mut budget),
            Ok(ext.as_bytes().to_vec())
        );
    }
}

#[cfg(unix)]
#[test]
fn symlink_with_image_name_to_non_image_is_not_image() {
    use std::os::unix::fs::symlink;
    let doc = doc_in("sc005-unix").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("notes.txt"), b"text").unwrap();
    fs::create_dir(dir.join("sub")).unwrap();
    symlink("notes.txt", dir.join("fake.png")).unwrap();
    symlink("sub", dir.join("subdir.png")).unwrap();

    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&doc, "fake.png", &mut budget),
        Err(AssetError::NotImage)
    );
    assert_eq!(
        read_asset(&doc, "subdir.png", &mut budget),
        Err(AssetError::NotImage)
    );
    assert_eq!(budget.used(), 0);
}

#[test]
fn missing_document_or_directory_is_io() {
    let doc = doc_in("missing").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("a.png"), b"a").unwrap();
    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&dir.join("gone.md"), "a.png", &mut budget),
        Err(AssetError::Io)
    );
    assert_eq!(
        read_asset(&doc, "nodir/a.png", &mut budget),
        Err(AssetError::Io)
    );
    assert_eq!(budget.used(), 0);
}

#[test]
fn file_size_limit() {
    let doc = doc_in("sc006").unwrap();
    let dir = doc.parent().unwrap();
    write(&dir.join("big.png"), &filled(MAX_ASSET_BYTES + 1, 1)).unwrap();
    write(&dir.join("ok.png"), &filled(MAX_ASSET_BYTES, 2)).unwrap();

    let mut budget = AssetBudget::new();
    assert_eq!(
        read_asset(&doc, "big.png", &mut budget),
        Err(AssetError::TooLarge)
    );
    assert_eq!(budget.used(), 0);
    let ok = read_asset(&doc, "ok.png", &mut budget).unwrap();
    assert_eq!(ok.len() as u64, MAX_ASSET_BYTES);
    assert!(ok.iter().all(|&b| b == 2));
    assert_eq!(budget.used(), MAX_ASSET_BYTES);
}

#[test]
fn budget_counts_unique_files() {
    const SIZE: u64 = 4 * 1024 * 1024 + 1;
    let doc = doc_in("sc007").unwrap();
    let dir = doc.parent().unwrap();
    for i in 1..=5u8 {
        write(&dir.join(format!("p{i}.png")), &filled(SIZE, i)).unwrap();
    }

    let mut budget = AssetBudget::new();
    for i in 1..=4u8 {
        let bytes = read_asset(&doc, &format!("p{i}.png"), &mut budget).unwrap();
        assert_eq!(bytes, filled(SIZE, i));
    }
    assert_eq!(
        read_asset(&doc, "p5.png", &mut budget),
        Err(AssetError::BudgetExceeded)
    );
    assert_eq!(budget.used(), 4 * SIZE);
    assert_eq!(read_asset(&doc, "p1.png", &mut budget), Ok(filled(SIZE, 1)));
    assert_eq!(
        read_asset(&doc, "./p1.png", &mut budget),
        Ok(filled(SIZE, 1))
    );
    assert_eq!(budget.used(), 4 * SIZE);
    // Отказы бюджет не меняют
    assert_eq!(
        read_asset(&doc, "../x.png", &mut budget),
        Err(AssetError::Escape)
    );
    assert_eq!(
        read_asset(&doc, "missing.png", &mut budget),
        Err(AssetError::Io)
    );
    assert_eq!(budget.used(), 4 * SIZE);
}

#[test]
fn budget_boundary_is_inclusive() {
    let doc = doc_in("budget-boundary").unwrap();
    let dir = doc.parent().unwrap();
    for i in 1..=4u8 {
        write(&dir.join(format!("q{i}.png")), &filled(MAX_ASSET_BYTES, i)).unwrap();
    }
    write(&dir.join("tiny.png"), b"1").unwrap();
    write(&dir.join("empty.png"), b"").unwrap();

    let mut budget = AssetBudget::default();
    assert_eq!(budget.used(), 0);
    for i in 1..=4u8 {
        assert!(read_asset(&doc, &format!("q{i}.png"), &mut budget).is_ok());
    }
    assert_eq!(budget.used(), MAX_BUDGET_BYTES);
    assert_eq!(
        read_asset(&doc, "tiny.png", &mut budget),
        Err(AssetError::BudgetExceeded)
    );
    assert_eq!(read_asset(&doc, "empty.png", &mut budget), Ok(Vec::new()));
    assert_eq!(budget.used(), MAX_BUDGET_BYTES);
    // Новый бюджет — сброс
    let mut fresh = AssetBudget::new();
    assert_eq!(read_asset(&doc, "tiny.png", &mut fresh), Ok(b"1".to_vec()));
    assert_eq!(fresh.used(), 1);
}

#[test]
fn errors_display_distinct_russian_text() {
    let all = [
        AssetError::BadEncoding,
        AssetError::Scheme,
        AssetError::Absolute,
        AssetError::Home,
        AssetError::Escape,
        AssetError::NotImage,
        AssetError::TooLarge,
        AssetError::BudgetExceeded,
        AssetError::Io,
    ];
    let texts: Vec<String> = all.iter().map(ToString::to_string).collect();
    for (err, text) in all.iter().zip(&texts) {
        assert!(!text.is_empty(), "{err:?}");
        assert!(
            text.chars().any(|c| ('\u{400}'..='\u{4FF}').contains(&c)),
            "{err:?}: {text}"
        );
        let dyn_err: &dyn std::error::Error = err;
        assert_eq!(dyn_err.to_string(), *text);
        assert!(dyn_err.source().is_none());
    }
    for (i, a) in texts.iter().enumerate() {
        for b in &texts[i + 1..] {
            assert_ne!(a, b);
        }
    }
}
