#![forbid(unsafe_code)]
fn options(o: &mut comrak::Options, on: bool) {
    // ruleid: nimmd-comrak-raw-html
    o.render.unsafe_ = true;
    // ruleid: nimmd-comrak-raw-html
    let _ = comrak::RenderOptions { r#unsafe: true };
    // ruleid: nimmd-comrak-raw-html
    o.render.unsafe_ = on;
    // ruleid: nimmd-comrak-raw-html
    o.render.r#unsafe = !false;
    // ruleid: nimmd-comrak-raw-html
    o.render.unsafe_ = false || on;
    // ruleid: nimmd-comrak-raw-html
    let _ = comrak::RenderOptions { unsafe_: on, ..Default::default() };
    // ok: nimmd-comrak-raw-html
    o.render.unsafe_ = false;
    // ok: nimmd-comrak-raw-html
    let _ = comrak::RenderOptions { r#unsafe: false, ..Default::default() };
    // ok: nimmd-comrak-raw-html
    let _ = o.render.unsafe_ == on;
}
