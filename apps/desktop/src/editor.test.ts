import { ensureSyntaxTree } from "@codemirror/language";
import { runScopeHandlers, type EditorView } from "@codemirror/view";
import { afterEach, expect, test, vi } from "vitest";
import { mountEditor } from "./editor";

vi.hoisted(() => {
  Object.defineProperty(navigator, "platform", {
    value: "MacIntel",
    configurable: true,
  });
});

let view: EditorView | undefined;

afterEach(() => {
  view?.destroy();
  view = undefined;
  document.body.replaceChildren();
});

function open(source: string): EditorView {
  view = mountEditor(document.body, source);
  return view;
}

test("сохраняет Markdown как текст и нормализует только CRLF при чтении и вставке", () => {
  const editor = open("%%неизвестно%%\r\n[[ссылка]]\rстрока\n");
  const paste = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(paste, "clipboardData", {
    value: { getData: () => "a\r\nb\rc\n" },
  });
  editor.contentDOM.dispatchEvent(paste);
  expect(paste.defaultPrevented).toBe(true);
  expect(editor.state.lineBreak).toBe("\n");
  expect(editor.state.doc.toString()).toBe(
    "a\nb\rc\n%%неизвестно%%\n[[ссылка]]\rстрока\n",
  );
});

test("разбирает YAML, GFM и расширения NimMD поверх исходного текста", () => {
  const source = [
    "---",
    "markdown: obsidian",
    "---",
    "",
    "| a | b |",
    "| - | - |",
    "| 1 | 2 |",
    "",
    "- [ ] ~~дело~~ https://example.com",
    "",
    "> [!nOtE]",
    "> Текст",
    "",
    "Цена $5 и $10. Формула $x+y$ и [[страница|ссылка]].",
    "",
    "$$",
    "x^2",
    "$$",
    "",
    "Сноска [^x].",
    "",
    "[^x]: текст",
    "",
    "> [!abstract]",
    "",
    "`$x$ [[literal]]` [ссылка](https://example.com)",
  ].join("\n");
  const { state } = open(source);
  const nodes: string[] = [];
  ensureSyntaxTree(state, source.length, 1000)?.iterate({
    enter(node) {
      nodes.push(node.name);
    },
  });
  for (const name of [
    "Frontmatter",
    "Table",
    "Task",
    "Strikethrough",
    "URL",
    "AlertMarker",
    "InlineMath",
    "DisplayMath",
    "WikiLink",
    "FootnoteReference",
    "FootnoteDefinition",
    "InlineCode",
    "Link",
  ]) {
    expect(nodes).toContain(name);
  }
  expect(nodes.filter((name) => name === "InlineMath")).toHaveLength(1);
  expect(nodes.filter((name) => name === "WikiLink")).toHaveLength(1);
  expect(nodes.filter((name) => name === "AlertMarker")).toHaveLength(1);
  expect(state.doc.toString()).toBe(source);
});

test("показывает исходник на активной строке и переключает декорации Cmd+/", () => {
  const source = "**первая**\n**вторая**";
  const editor = open(source);
  const lines = () =>
    Array.from(
      editor.contentDOM.querySelectorAll(".cm-line"),
      (line) => line.textContent,
    );
  const toggle = () =>
    runScopeHandlers(
      editor,
      new KeyboardEvent("keydown", { key: "/", metaKey: true }),
      "editor",
    );

  expect(lines()).toEqual(["**первая**", "вторая"]);
  editor.dispatch({ selection: { anchor: source.length } });
  expect(lines()).toEqual(["первая", "**вторая**"]);
  expect(toggle()).toBe(true);
  expect(lines()).toEqual(["**первая**", "**вторая**"]);
  expect(toggle()).toBe(true);
  expect(lines()).toEqual(["первая", "**вторая**"]);
  expect(editor.state.doc.toString()).toBe(source);
});

test("распознаёт алерт только при открытии цитаты и допускает заголовок", () => {
  const source = [
    "> [!NOTE] Заголовок",
    "> тело",
    "",
    "> # Обычная цитата",
    "> [!TIP]",
    "> тело",
    "",
    "> Обычная цитата",
    ">",
    "> [!WARNING]",
    "> тело",
    "",
    "> [!CAUTION]- Сворачиваемый callout",
    "> тело",
    "",
    ">[!IMPORTANT]",
    "",
    ">  [!TIP]",
  ].join("\n");
  const { state } = open(source);
  const markers: string[] = [];
  ensureSyntaxTree(state, source.length, 1000)?.iterate({
    enter({ name, from, to }) {
      if (name === "AlertMarker") markers.push(source.slice(from, to));
    },
  });
  expect(markers).toEqual(["[!NOTE]"]);
});

test("сохраняет абзацы сноски и многострочную формулу в синтаксическом дереве", () => {
  const source =
    "Сноска [^x] и $x\n+y$.\n\n[^x]: первый\n\n    второй\n\nДругой абзац.";
  const { state } = open(source);
  const nodes = new Map<string, string>();
  ensureSyntaxTree(state, source.length, 1000)?.iterate({
    enter({ name, from, to }) {
      if (name === "FootnoteDefinition" || name === "InlineMath")
        nodes.set(name, source.slice(from, to));
    },
  });
  expect(nodes.get("FootnoteDefinition")).toBe("[^x]: первый\n\n    второй");
  expect(nodes.get("InlineMath")).toBe("$x\n+y$");
});

test("разбирает Markdown под незакрытой строкой YAML и обновляет разбор после правок", () => {
  const editor = open("---");
  const nodes = () => {
    const names: string[] = [];
    ensureSyntaxTree(editor.state, editor.state.doc.length, 1000)?.iterate({
      enter(node) {
        names.push(node.name);
      },
    });
    return names;
  };
  expect(nodes()).toContain("HorizontalRule");
  editor.dispatch({
    changes: { from: 3, insert: "\n# Заголовок\n\n**текст**" },
  });
  expect(nodes()).toContain("ATXHeading1");
  expect(nodes()).toContain("StrongEmphasis");
  expect(nodes()).not.toContain("Frontmatter");

  editor.dispatch({ changes: { from: 4, insert: "title: Заголовок\n---\n" } });
  expect(nodes()).toContain("Frontmatter");
  expect(nodes()).toContain("StrongEmphasis");

  editor.dispatch({ changes: { from: 4, to: editor.state.doc.line(4).from } });
  expect(nodes()).toContain("ATXHeading1");
  expect(nodes()).not.toContain("Frontmatter");

  editor.dispatch({ changes: { from: 4, insert: "--x-\n" } });
  expect(nodes()).not.toContain("Frontmatter");
  editor.dispatch({ changes: { from: 6, to: 7 } });
  expect(nodes()).toContain("Frontmatter");
  expect(nodes()).toContain("ATXHeading1");
});
