fn options(o: &mut comrak::Options) {
    // ruleid: nimmd-comrak-raw-html
    o.render.unsafe_ = true;
    // ruleid: nimmd-comrak-raw-html
    let _ = comrak::RenderOptions { r#unsafe: true };
    // ok: nimmd-comrak-raw-html
    o.render.unsafe_ = false;
}
