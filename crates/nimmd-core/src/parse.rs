//! Опции разбора comrak, общие для детектора и рендера.

use comrak::Options;

/// Режим надмножества: все расширения, которые встречаются в поддерживаемых диалектах.
///
/// Сырой HTML не пропускается (`render.unsafe_ = false`): документ недоверенный.
pub(crate) fn options() -> Options<'static> {
    let mut options = Options::default();
    let ext = &mut options.extension;
    ext.table = true;
    ext.tasklist = true;
    ext.strikethrough = true;
    ext.footnotes = true;
    ext.autolink = true;
    ext.alerts = true;
    ext.math_dollars = true;
    ext.wikilinks_title_after_pipe = true;
    ext.front_matter_delimiter = Some("---".to_owned());
    options.render.r#unsafe = false;
    options
}
