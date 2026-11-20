//! Адаптер тестов редактора. Читает Markdown-фикстуры из JSON, возвращает HTML и проекцию AST.

use std::{error::Error, io};

use comrak::{
    Arena,
    nodes::{ListType, Node, NodeValue, TableAlignment},
    parse_document,
};
use nimmd_core::{RenderOptions, render, text};
use serde_json::{Value, json};

// Тест использует те же опции comrak, что рендер ядра.
#[path = "../src/parse.rs"]
mod parse;

fn main() -> Result<(), Box<dyn Error>> {
    let fixtures: Vec<String> = serde_json::from_reader(io::stdin().lock())?;
    let documents: Vec<_> = fixtures
        .iter()
        .map(|source| {
            let arena = Arena::new();
            let (_, markdown) = text::split_frontmatter(source);
            let root = parse_document(&arena, markdown, parse::options());
            json!({
                "body": render(source, &RenderOptions::default()).body,
                "tree": project(root),
            })
        })
        .collect();
    serde_json::to_writer(io::stdout().lock(), &documents)?;
    Ok(())
}

fn descendants_text(node: Node<'_>) -> String {
    let mut result = String::new();
    for child in node.descendants() {
        match &child.data().value {
            NodeValue::Text(text) => result.push_str(text),
            NodeValue::Code(code) => result.push_str(&code.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => result.push('\n'),
            _ => {}
        }
    }
    result
}

fn paragraph_text(node: Node<'_>) -> String {
    node.children()
        .find(|child| matches!(child.data().value, NodeValue::Paragraph))
        .map_or_else(String::new, descendants_text)
}

fn project(node: Node<'_>) -> Vec<Value> {
    let children = || node.children().flat_map(project).collect::<Vec<_>>();
    vec![match &node.data().value {
        NodeValue::Document | NodeValue::Paragraph => return children(),
        NodeValue::Heading(heading) => {
            json!({"kind": "heading", "level": heading.level, "text": descendants_text(node)})
        }
        NodeValue::List(list) => {
            let ordered = list.list_type == ListType::Ordered;
            let start = if ordered { list.start } else { 0 };
            json!({"kind": "list", "ordered": ordered, "start": start, "children": children()})
        }
        NodeValue::Item(_) => {
            json!({"kind": "item", "checked": null, "text": paragraph_text(node), "children": children()})
        }
        NodeValue::TaskItem(task) => {
            json!({"kind": "item", "checked": task.symbol.is_some(), "text": paragraph_text(node), "children": children()})
        }
        NodeValue::Table(table) => {
            let alignment: Vec<_> = table
                .alignments
                .iter()
                .map(|alignment| match alignment {
                    TableAlignment::None => "none",
                    TableAlignment::Left => "left",
                    TableAlignment::Center => "center",
                    TableAlignment::Right => "right",
                })
                .collect();
            json!({"kind": "table", "alignment": alignment, "children": children()})
        }
        NodeValue::TableRow(header) => {
            json!({"kind": "row", "header": *header, "children": children()})
        }
        NodeValue::TableCell => json!({"kind": "cell", "text": descendants_text(node)}),
        NodeValue::CodeBlock(code) => json!({
            "kind": "code",
            "language": code.info.split_whitespace().next().unwrap_or_default(),
            "text": code.literal.strip_suffix('\n').unwrap_or(&code.literal),
        }),
        NodeValue::Alert(alert) => json!({
            "kind": "alert",
            "type": alert.alert_type.default_title().to_ascii_lowercase(),
            "text": paragraph_text(node),
            "children": children(),
        }),
        NodeValue::BlockQuote => {
            json!({"kind": "quote", "text": paragraph_text(node), "children": children()})
        }
        NodeValue::Image(image) => json!({"kind": "image", "url": image.url}),
        NodeValue::Math(math) => {
            json!({"kind": "math", "display": math.display_math, "text": math.literal.trim()})
        }
        // Исключения из сравнения перечислены в editor-contract.test.ts.
        _ => return Vec::new(),
    }]
}
