//! Интеграционные тесты C ABI: UTF-8, картинки, вложенность, cbindgen.

use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::ptr;
use std::slice;

use nimmd_ffi::{
    NIMMD_RENDER_OPTIONS_VERSION, NimmdBuffer, NimmdRenderOptions, nimmd_buffer_free,
    nimmd_read_asset, nimmd_render_html,
};

fn opts_default() -> NimmdRenderOptions {
    NimmdRenderOptions {
        version: NIMMD_RENDER_OPTIONS_VERSION,
        theme: 0,
        math: 0,
        assets: 0,
        remote_images: 0,
        max_bytes: 0,
        base_url: ptr::null(),
        extension: ptr::null(),
    }
}

fn render(src: &[u8], opts: &NimmdRenderOptions) -> *mut NimmdBuffer {
    // SAFETY: `src`/`opts` живы на время вызова.
    unsafe { nimmd_render_html(src.as_ptr(), src.len(), ptr::null(), ptr::from_ref(opts)) }
}

fn html_of(buf: *mut NimmdBuffer) -> String {
    assert!(!buf.is_null(), "ожидался буфер HTML");
    // SAFETY: буфер только что вернул nimmd_render_html.
    let (data, len) = unsafe { ((*buf).data, (*buf).len) };
    let bytes = if len == 0 || data.is_null() {
        &[][..]
    } else {
        // SAFETY: data/len принадлежат валидному NimmdBuffer.
        unsafe { slice::from_raw_parts(data, len) }
    };
    // HTML из ядра — UTF-8; lossy на случай битого буфера в тесте.
    let s = String::from_utf8_lossy(bytes).into_owned();
    // SAFETY: парный free к render/read.
    unsafe { nimmd_buffer_free(buf) };
    s
}

#[test]
fn render_html_markdown_title() {
    let opts = opts_default();
    let html = html_of(render(b"# Hello\n\npara\n", &opts));
    assert!(html.contains("<title>Hello</title>"), "{html}");
    assert!(html.contains("<main class=\"nimmd\">"), "{html}");
}

#[test]
fn render_html_from_utf8_lossy_sets_not_utf8_banner() {
    // Невалидный UTF-8: C0 — продолжение без стартового байта.
    let opts = opts_default();
    let html = html_of(render(b"# X\n\n\xC0\x80\n", &opts));
    assert!(html.contains("not-utf8"), "{html}");
    assert!(html.contains('\u{FFFD}') || html.contains("�"), "{html}");
}

#[test]
fn render_html_rejects_unknown_options_version() {
    let mut opts = opts_default();
    opts.version = 999;
    let buf = render(b"# x\n", &opts);
    assert!(buf.is_null());
}

#[test]
fn render_html_extension_json() {
    let ext = CString::new("json").unwrap();
    let mut opts = opts_default();
    opts.extension = ext.as_ptr();
    let html = html_of(render(b"{\"a\":1}\n", &opts));
    assert!(html.contains("nimmd-data"), "{html}");
    assert!(html.contains("data-kind=\"json\""), "{html}");
}

#[test]
fn read_asset_returns_bytes_and_rejects_escape() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("nimmd-ffi-assets");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("img")).unwrap();
    let doc = root.join("note.md");
    fs::write(&doc, b"# n\n").unwrap();
    fs::write(root.join("img/a.png"), b"PNGDATA").unwrap();

    let doc_c = CString::new(doc.to_str().unwrap()).unwrap();
    let href = CString::new("img/a.png").unwrap();
    // SAFETY: C-строки живы до free.
    let buf = unsafe { nimmd_read_asset(doc_c.as_ptr(), href.as_ptr()) };
    assert!(!buf.is_null());
    // SAFETY: валидный буфер от read_asset.
    unsafe {
        let got = slice::from_raw_parts((*buf).data, (*buf).len);
        assert_eq!(got, b"PNGDATA");
        nimmd_buffer_free(buf);
    }

    let bad = CString::new("../secret.png").unwrap();
    // SAFETY: C-строки валидны.
    let denied = unsafe { nimmd_read_asset(doc_c.as_ptr(), bad.as_ptr()) };
    assert!(denied.is_null());
}

#[test]
fn nesting_depth_10_000_does_not_abort() {
    let opts = opts_default();
    let src = format!("{}x\n", ">".repeat(10_000));
    let html = html_of(render(src.as_bytes(), &opts));
    assert!(html.contains("<!doctype html>"), "{html}");
    assert!(html.contains("<main class=\"nimmd\">"), "{html}");
}

#[test]
fn cbindgen_header_lists_exports() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let header = fs::read_to_string(crate_dir.join("include/nimmd.h")).unwrap_or_default();
    assert!(
        !header.is_empty(),
        "include/nimmd.h должен появиться из build.rs"
    );
    for needle in [
        "NimmdBuffer",
        "NimmdRenderOptions",
        "nimmd_render_html",
        "nimmd_read_asset",
        "nimmd_buffer_free",
        "NIMMD_RENDER_OPTIONS_VERSION",
    ] {
        assert!(
            header.contains(needle),
            "нет {needle} в заголовке:\n{header}"
        );
    }
}
