import { forceParsing, syntaxTreeAvailable } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, test, vi } from "vitest";
import { blockWidgets, markdownSyntax } from "./widgets";

const table = "| Имя | Значение |\n| --- | --- |\n| А\\|Б | <img src=x> |";
const diagram = "```Mermaid title\nflowchart TD\n  A --> B\n```";
let editor: EditorView | undefined;

function create(doc: string, render = vi.fn()) {
  editor = new EditorView({ doc, parent: document.body, extensions: [markdownSyntax, blockWidgets(render)] });
  return editor;
}

afterEach(() => { editor?.destroy(); });

test("таблица заменяет блок, но при выделении показывает редактируемый исходник", () => {
  const doc = `Вступление\n\n${table}\n\nКонец`;
  const view = create(doc);
  const rendered = view.dom.querySelector("table");
  expect(rendered?.querySelector("th")?.textContent).toBe("Имя");
  expect([...view.dom.querySelectorAll("td")].map(cell => cell.textContent)).toEqual(["А|Б", "<img src=x>"]);
  expect(view.dom.querySelector("img")).toBeNull();

  const from = doc.indexOf(table);
  view.dispatch({ selection: { anchor: 0, head: from + 1 } });
  expect(view.dom.querySelector("table")).toBeNull();
  expect(view.contentDOM.textContent).toContain("| Имя | Значение |");
  view.dispatch({ changes: { from: from + 2, to: from + 5, insert: "Ключ" }, selection: { anchor: 0 } });
  expect(view.dom.querySelector("th")?.textContent).toBe("Ключ");
  expect(view.state.doc.toString()).toBe(doc.replace("Имя", "Ключ"));
});

test("клик открывает исходник таблицы после сдвига её диапазона", () => {
  const view = create(`Начало\n\n${table}`);
  const cell = view.contentDOM.querySelector("th");
  expect(cell).not.toBeNull();
  view.dispatch({ changes: { from: 0, insert: "!" } });
  expect(view.contentDOM.querySelector("th")).toBe(cell);
  cell?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true, button: 0 }));
  expect(view.dom.querySelector("table")).toBeNull();
  expect(view.state.selection.main.anchor).toBe(view.state.doc.line(3).from);
});

test("таблица сохраняет пустые и недостающие ячейки на своих местах", () => {
  const view = create("До таблицы\n\n  A | B | C\n  --- | --- | ---\n  | 1 || 3 |  \n  | 4 |\n\nПосле таблицы");
  const rows = [...view.dom.querySelectorAll("tr")];
  expect(rows.map(row => [...row.cells].map(cell => cell.textContent))).toEqual([
    ["A", "B", "C"], ["1", "", "3"], ["4", "", ""],
  ]);
});

test("закрытый Mermaid получает виджет после исходника и сохраняет DOM при соседней правке", () => {
  const render = vi.fn();
  const view = create(`Начало\n\n${diagram}\n\nКонец`, render);
  const preview = view.dom.querySelector(".mermaid");
  expect(preview).not.toBeNull();
  expect(view.contentDOM.textContent).toContain("```Mermaid title");
  expect(render).toHaveBeenCalledExactlyOnceWith(preview, "flowchart TD\n  A --> B");

  view.dispatch({ changes: { from: 1, insert: "!" } });
  expect(view.dom.querySelector(".mermaid")).toBe(preview);
  expect(render).toHaveBeenCalledTimes(1);
  const from = view.state.doc.toString().indexOf("A --> B");
  view.dispatch({ changes: { from, to: from + 1, insert: "C" } });
  expect(render).toHaveBeenLastCalledWith(view.dom.querySelector(".mermaid"), "flowchart TD\n  C --> B");
});

test.each([
  ["незакрытый", "```mermaid\nflowchart TD"],
  ["чужой", "```mermaidx\nflowchart TD\n```"],
  ["пустой", "```mermaid\n```"],
  ["слишком длинный", `\`\`\`mermaid\n${"x".repeat(8193)}\n\`\`\``],
])("%s блок кода остаётся текстом", (_description, doc) => {
  const render = vi.fn();
  const view = create(doc, render);
  expect(forceParsing(view, doc.length, 1000)).toBe(true);
  expect(view.dom.querySelector(".mermaid")).toBeNull();
  expect(render).not.toHaveBeenCalled();
});

test("фоновое продолжение разбора добавляет декорации без изменения текста", () => {
  const doc = `${"Абзац текста.\n\n".repeat(2000)}${table}\n\nКонец`;
  const view = create(doc);
  expect(syntaxTreeAvailable(view.state)).toBe(false);
  expect(forceParsing(view, doc.length, 1000)).toBe(true);
  const from = doc.indexOf(table);
  view.dispatch({ effects: EditorView.scrollIntoView(from) });
  const ranges: number[] = [];
  for (const set of view.state.facet(EditorView.decorations)) {
    if (typeof set !== "function") set.between(from, doc.length, (start) => { ranges.push(start); });
  }
  expect(ranges).toEqual([from]);
  expect(view.state.doc.toString()).toBe(doc);
});
