use std::ffi::CString;
use std::{fs, ptr, slice};

use super::{
    NIMMD_KIND_JSON, NIMMD_THEME_DARK, NimmdBuffer, NimmdRenderOptions, nimmd_asset_budget_free,
    nimmd_asset_budget_open, nimmd_buffer_free, nimmd_read_asset, nimmd_read_asset_with_type,
    nimmd_render_html, nimmd_render_options_v1_default, render_boundary, render_on_worker,
};

fn take_buffer(buffer: NimmdBuffer) -> Vec<u8> {
    if buffer.ptr.is_null() {
        assert_eq!(buffer.len, 0);
        return Vec::new();
    }
    // SAFETY: буфер возвращён текущим вызовом ABI и ещё не освобождён.
    let bytes = unsafe { slice::from_raw_parts(buffer.ptr, buffer.len) }.to_vec();
    // SAFETY: владение исходным буфером передаётся обратно один раз.
    unsafe { nimmd_buffer_free(buffer) };
    bytes
}

fn render(bytes: &[u8], options: &NimmdRenderOptions) -> String {
    // SAFETY: срез и опции живут до возврата из функции ABI.
    let buffer = unsafe { nimmd_render_html(bytes.as_ptr(), bytes.len(), options) };
    String::from_utf8(take_buffer(buffer)).unwrap()
}

#[test]
fn renders_heading_and_blocks_remote_images_by_default() {
    let html = render(
        b"# NimMD\n\n![image](https://example.invalid/image.png)",
        &nimmd_render_options_v1_default(),
    );
    assert!(html.contains("<h1>NimMD</h1>"));
    assert!(html.contains("class=\"remote-image\""));
    assert!(!html.contains("<img"));
}

#[test]
fn renders_named_data_format_in_selected_theme() {
    let options = NimmdRenderOptions {
        theme: NIMMD_THEME_DARK,
        kind: NIMMD_KIND_JSON,
        ..nimmd_render_options_v1_default()
    };
    let html = render(br#"{"test":true}"#, &options);
    assert!(html.contains("data-theme=\"dark\""));
    assert!(html.contains("data-kind=\"json\""));
}

#[test]
fn invalid_utf8_is_replaced_and_reported() {
    let html = render(b"before\xffafter", &nimmd_render_options_v1_default());
    assert!(html.contains("class=\"banner not-utf8\""));
    assert!(html.contains("before\u{fffd}after"));
}

#[test]
fn clips_on_character_boundary_after_normalizing_line_endings() {
    let options = NimmdRenderOptions {
        max_bytes: 5,
        ..nimmd_render_options_v1_default()
    };
    let html = render("a\r\nb\r\nαtail".as_bytes(), &options);
    assert!(html.contains("class=\"banner truncated\""));
    assert!(!html.contains("αtail"));
    assert!(!html.contains("class=\"banner not-utf8\""));
}

#[test]
fn reports_invalid_utf8_at_eof_but_not_at_artificial_read_boundary() {
    let options = NimmdRenderOptions {
        max_bytes: 5,
        ..nimmd_render_options_v1_default()
    };
    let mut bytes = vec![b'a'; 17];
    // Восемнадцатый байт находится на границе префикса, который читает ABI.
    bytes.push(0xc2);
    let html = render(&bytes, &options);
    assert!(html.contains("class=\"banner not-utf8\""));

    bytes.push(0xa2);
    let html = render(&bytes, &options);
    assert!(!html.contains("class=\"banner not-utf8\""));
}

#[test]
fn unsupported_version_does_not_read_the_rest_of_options() {
    let version = u32::MAX;
    let bytes = b"# Directly from Rust";
    // SAFETY: неподдерживаемая версия передаёт только начальное поле ABI.
    let buffer =
        unsafe { nimmd_render_html(bytes.as_ptr(), bytes.len(), ptr::from_ref(&version).cast()) };
    let html = String::from_utf8(take_buffer(buffer)).unwrap();
    assert!(html.contains("Не удалось показать документ"));
    assert!(!html.contains("Directly from Rust"));
}

#[test]
fn null_with_zero_length_represents_an_empty_document() {
    // SAFETY: нулевые указатели допустимы для пустого документа и опций по умолчанию.
    let buffer = unsafe { nimmd_render_html(ptr::null(), 0, ptr::null()) };
    let html = String::from_utf8(take_buffer(buffer)).unwrap();
    assert!(html.contains("<main class=\"nimmd\"></main>"));
}

#[test]
fn null_with_nonzero_length_returns_error_document() {
    // SAFETY: функция проверяет нулевой указатель до обращения к документу.
    let buffer = unsafe { nimmd_render_html(ptr::null(), 1, ptr::null()) };
    let html = String::from_utf8(take_buffer(buffer)).unwrap();
    assert!(html.contains("Не удалось показать документ"));
}

#[test]
fn ten_thousand_nested_quotes_render_without_aborting() {
    let input = "> ".repeat(10_000) + "последняя цитата";
    let html = render(input.as_bytes(), &nimmd_render_options_v1_default());
    assert_eq!(html.matches("<blockquote>").count(), 64);
    assert!(html.contains("последняя цитата"));
}

#[test]
fn worker_panic_is_converted_to_html_even_in_release() {
    let buffer = render_boundary(|| render_on_worker(|| panic!("тестовая паника")));
    let html = String::from_utf8(take_buffer(buffer)).unwrap();
    assert!(html.contains("Не удалось показать документ"));
    assert!(!html.contains("тестовая паника"));
}

#[test]
fn asset_reader_checks_paths_from_the_document_directory() {
    let dir = tempfile::tempdir().unwrap();
    let document_dir = dir.path().join("readme");
    fs::create_dir(&document_dir).unwrap();
    let document = document_dir.join("readme.md");
    fs::write(&document, "# NimMD").unwrap();
    fs::write(document_dir.join("img.png"), b"local image").unwrap();
    fs::write(dir.path().join("img.png"), b"different image").unwrap();
    let document = document.canonicalize().unwrap();
    let document = CString::new(document.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: канонический путь документа доступен до возврата.
    let budget = unsafe { nimmd_asset_budget_open(document.as_ptr()) };
    assert!(!budget.is_null());
    for (href, expected) in [("img.png", &b"local image"[..]), ("../img.png", &[][..])] {
        let href = CString::new(href).unwrap();
        // SAFETY: строки и действительный бюджет живут до возврата. Вызовы последовательны.
        let buffer = unsafe { nimmd_read_asset(document.as_ptr(), href.as_ptr(), budget) };
        assert_eq!(take_buffer(buffer), expected);
    }
    // SAFETY: бюджет получен из ABI, освобождается один раз после чтения.
    unsafe { nimmd_asset_budget_free(budget) };
}

#[cfg(unix)]
#[test]
fn asset_reader_returns_the_type_of_the_file_behind_an_alias() {
    let dir = tempfile::tempdir().unwrap();
    let document = dir.path().join("readme.md");
    fs::write(&document, "![Рисунок](picture)").unwrap();
    let document = document.canonicalize().unwrap();
    let document = CString::new(document.as_os_str().as_encoded_bytes()).unwrap();
    fs::write(dir.path().join("graphic.svg"), b"<svg/>").unwrap();
    std::os::unix::fs::symlink("graphic.svg", dir.path().join("picture")).unwrap();

    // SAFETY: канонический путь документа доступен до возврата.
    let budget = unsafe { nimmd_asset_budget_open(document.as_ptr()) };
    assert!(!budget.is_null());
    // SAFETY: строки и действительный бюджет живут до возврата.
    let image =
        unsafe { nimmd_read_asset_with_type(document.as_ptr(), c"picture".as_ptr(), budget) };
    assert!(!image.content_type.is_null());
    assert_eq!(take_buffer(image.buffer), b"<svg/>");
    // SAFETY: ненулевой content_type — статическая строка C из ABI.
    let content_type = unsafe { std::ffi::CStr::from_ptr(image.content_type) };
    assert_eq!(content_type, c"image/svg+xml");

    // SAFETY: обе строки и бюджет принадлежат текущему документу и доступны.
    let missing =
        unsafe { nimmd_read_asset_with_type(document.as_ptr(), c"missing.svg".as_ptr(), budget) };
    assert!(missing.content_type.is_null());
    assert_eq!(take_buffer(missing.buffer), b"");

    // SAFETY: бюджет получен из ABI, освобождается один раз после чтения.
    unsafe { nimmd_asset_budget_free(budget) };
}

#[cfg(unix)]
#[test]
fn asset_reader_keeps_the_original_directory_when_its_path_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let original = root.join("original");
    let original_doc = original.join("readme.md");
    fs::create_dir(&original).unwrap();
    fs::write(&original_doc, "![Рисунок](img.png)").unwrap();
    fs::write(original.join("img.png"), b"original image").unwrap();
    let document = CString::new(original_doc.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: канонический путь документа доступен до возврата.
    let budget = unsafe { nimmd_asset_budget_open(document.as_ptr()) };
    assert!(!budget.is_null());

    let moved = root.join("moved");
    fs::rename(&original, &moved).unwrap();
    fs::create_dir(&original).unwrap();
    fs::write(&original_doc, "Заменённый документ").unwrap();
    fs::write(original.join("img.png"), b"replacement image").unwrap();
    // SAFETY: строки и действительный бюджет живут до возврата.
    let image =
        unsafe { nimmd_read_asset_with_type(document.as_ptr(), c"img.png".as_ptr(), budget) };
    assert_eq!(take_buffer(image.buffer), b"original image");

    let different_doc = moved.join("readme.md");
    let different_doc = CString::new(different_doc.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: все указатели действительны. Функция проверяет соответствие пути бюджету.
    let refused = unsafe { nimmd_read_asset(different_doc.as_ptr(), c"img.png".as_ptr(), budget) };
    assert_eq!(take_buffer(refused), b"");

    // SAFETY: бюджет получен из ABI, освобождается один раз после чтения.
    unsafe { nimmd_asset_budget_free(budget) };
}
