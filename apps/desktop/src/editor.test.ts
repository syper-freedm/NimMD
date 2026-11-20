import { ensureSyntaxTree } from "@codemirror/language";
import { runScopeHandlers, type EditorView } from "@codemirror/view";
import { clearMocks, mockConvertFileSrc, mockIPC } from "@tauri-apps/api/mocks";
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
  clearMocks();
});

function open(source: string, windowLabel?: string): EditorView {
  view = mountEditor(document.body, source, { windowLabel });
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

test("оформляет разметку и меняет только маркер задачи при клике", () => {
  const source =
    "Первая\n\n## Заголовок\n\n*курсив* ~~зачёркнутый~~ `код` [[ссылка]] [^x]\n\n- [ ] задача\n\n> [!NOTE]\n> заметка\n\n> [!abstract]\n> цитата\n\n[^x]: сноска";
  const editor = open(source);
  const dom = editor.contentDOM;
  for (const selector of [
    ".cm-heading-2",
    "em",
    "s",
    "code",
    ".cm-wikilink",
    ".footnote-ref",
    ".contains-task-list.task-list-item",
    ".markdown-alert.markdown-alert-note",
    ".markdown-alert-title",
    ".cm-quote",
    ".footnotes",
  ]) {
    expect(dom.querySelector(selector), selector).not.toBeNull();
  }
  expect(dom.querySelector(".cm-quote")?.textContent).toContain("[!abstract]");
  expect(dom.querySelectorAll(".markdown-alert-title")).toHaveLength(1);
  const checkbox = dom.querySelector<HTMLInputElement>(
    ".task-list-item-checkbox",
  );
  checkbox?.click();
  expect(editor.state.doc.toString()).toBe(source.replace("[ ]", "[x]"));
  expect(
    dom.querySelector<HTMLInputElement>(".task-list-item-checkbox")?.checked,
  ).toBe(true);
});

test("показывает таблицу, frontmatter и безопасные картинки, не меняя текст", () => {
  mockConvertFileSrc("macos");
  const source =
    "---\ntitle: Тест\n---\n\n| a | b |\n| :- | -: |\n| 1 | 2 |\n\n![локальная](./pic.png)\n\n![удалённая](https://example.com/pic.png)\n\nПоследняя";
  const editor = open(source, "nimmd-window-3");
  const dom = editor.contentDOM;
  const last = editor.state.doc.line(editor.state.doc.lines).from;
  editor.dispatch({ selection: { anchor: last } });
  expect(dom.querySelector(".frontmatter")?.textContent).toContain(
    "Frontmatter",
  );
  expect(dom.querySelector("tbody tr")?.textContent).toBe("12");
  expect(dom.querySelector<HTMLImageElement>("img")?.src).toContain(
    "path=.%2Fpic.png",
  );
  expect(dom.querySelectorAll("img")).toHaveLength(1);
  expect(dom.textContent).toContain("https://example.com/pic.png");
  editor.dispatch({ selection: { anchor: source.indexOf("| 1") } });
  expect(dom.querySelector("table")).toBeNull();
  expect(dom.textContent).toContain("| 1 | 2 |");
  dom.querySelector<HTMLButtonElement>(".frontmatter")?.click();
  expect(dom.querySelector(".frontmatter")).toBeNull();
  expect(dom.textContent).toContain("title: Тест");
  expect(editor.state.doc.toString()).toBe(source);
});

test("подсвечивает UTF-16-диапазон кода и YAML через syntect", async () => {
  const calls: unknown[] = [];
  mockIPC((command, args) => {
    expect(command).toBe("highlight");
    calls.push(args);
    const { code, lang } = args as { code: string; lang: string };
    const start = lang === "rust" ? code.indexOf("true") : 0;
    return [{ start, end: start + 4, class: "nimmd-test-scope" }];
  });
  const source = "---\ntest: 1\n---\n\n```rust\n/* 🦀 */ true\n```";
  const editor = open(source);
  editor.contentDOM.querySelector<HTMLButtonElement>(".frontmatter")?.click();
  await vi.waitFor(() => {
    expect(
      [...editor.contentDOM.querySelectorAll(".nimmd-test-scope")].map(
        (node) => node.textContent,
      ),
    ).toEqual(["test", "true"]);
  });
  expect(calls).toContainEqual({ lang: "rust", code: "/* 🦀 */ true\n" });
  expect(calls).toContainEqual({ lang: "yaml", code: "test: 1\n" });
  expect(editor.state.doc.toString()).toBe(source);
});

test("сообщение об ошибке KaTeX не становится частью Markdown", async () => {
  const parent = document.createElement("div");
  document.body.append(parent);
  const source = "Первая\n\nФормула $x$ здесь.";
  view = mountEditor(parent, source);
  const asset = await vi.waitFor(() => {
    const script = document.querySelector('script[src="/vendor/katex.min.js"]');
    expect(script).not.toBeNull();
    return script;
  });
  asset?.dispatchEvent(new Event("error"));
  await vi.waitFor(() => {
    expect(parent.textContent).toContain("Не удалось отрисовать формулу");
  });
  expect(view.state.doc.toString()).toBe(source);
  document.querySelector('link[href="/vendor/katex.min.css"]')?.remove();
});

test("открывает внешние ссылки через IPC и оставляет исходник неизменным", () => {
  const calls: unknown[] = [];
  mockIPC((command, value) => {
    expect(command).toBe("open_external");
    calls.push(value);
  });
  const source =
    "Первая\n\n[сайт](https://example.com/?q=test&amp;lang=ru) <mail@example.com> [локальный](file:///tmp/test)";
  const editor = open(source);
  const links = [...editor.contentDOM.querySelectorAll("a[href]")];
  expect(links.map((link) => link.textContent)).toEqual([
    "сайт",
    "mail@example.com",
  ]);
  for (const link of links) {
    expect(
      link.dispatchEvent(
        new MouseEvent("click", { bubbles: true, cancelable: true }),
      ),
    ).toBe(false);
  }
  expect(calls).toEqual([
    { url: "https://example.com/?q=test&lang=ru" },
    { url: "mailto:mail@example.com" },
  ]);
  expect(editor.state.doc.toString()).toBe(source);
});
