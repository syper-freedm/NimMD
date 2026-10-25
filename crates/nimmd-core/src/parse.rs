//! Опции разбора и рендера comrak, общие для детектора и рендера.

use std::sync::OnceLock;

use comrak::Options;

/// Режим надмножества: все расширения, которые встречаются в поддерживаемых диалектах.
///
/// Списки определений и id/атрибуты заголовков выключены (значения по умолчанию).
/// `front_matter_delimiter` не задан: frontmatter отрезает `split_frontmatter`, а блок `---…---`
/// в начале тела должен остаться в выводе. Сырой HTML не пропускается (`render.unsafe_ = false`),
/// а показывается экранированным текстом (`render.escape`): документ недоверенный.
pub(crate) fn options() -> &'static Options<'static> {
    static OPTIONS: OnceLock<Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut options = Options::default();
        let ext = &mut options.extension;
        ext.table = true;
        ext.tasklist = true;
        ext.strikethrough = true;
        ext.footnotes = true;
        ext.autolink = true;
        ext.alerts = true;
        ext.multiline_block_quotes = true;
        ext.math_dollars = true;
        ext.wikilinks_title_after_pipe = true;
        let render = &mut options.render;
        render.r#unsafe = false;
        render.escape = true;
        render.github_pre_lang = true;
        render.tasklist_classes = true;
        options
    })
}
