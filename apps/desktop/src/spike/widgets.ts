import { markdown } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import { StateField } from "@codemirror/state";
import type { EditorState, Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { Table } from "@lezer/markdown";

export const markdownSyntax = markdown({ extensions: Table, completeHTMLTags: false });
const maxDiagramLength = 8192;
type MarkdownNode = ReturnType<typeof syntaxTree>["topNode"];
type RenderDiagram = (node: HTMLElement, source: string) => void;

class TableWidget extends WidgetType {
  constructor(private readonly source: string, private readonly block: MarkdownNode) { super(); }

  eq(other: TableWidget) { return this.source === other.source; }

  toDOM(view: EditorView): HTMLElement {
    const table = view.dom.ownerDocument.createElement("table");
    const head = table.createTHead();
    const body = table.createTBody();
    let columns = 0;
    for (let row = this.block.firstChild; row; row = row.nextSibling) {
      const header = row.name === "TableHeader";
      if (!header && row.name !== "TableRow") continue;
      const tr = (header ? head : body).insertRow();
      const from = row.from - this.block.from;
      const to = row.to - this.block.from;
      const pipes = row.getChildren("TableDelimiter").map(pipe => pipe.from - this.block.from);
      if (pipes[0] !== from) pipes.unshift(from - 1);
      if (pipes.at(-1) !== from + this.source.slice(from, to).trimEnd().length - 1) pipes.push(to);
      if (header) columns = pipes.length - 1;
      for (let column = 0; column < columns; column++) {
        const cell = view.dom.ownerDocument.createElement(header ? "th" : "td");
        const left = pipes[column];
        const right = pipes[column + 1];
        if (left !== undefined && right !== undefined) {
          cell.textContent = this.source.slice(left + 1, right).trim().replaceAll("\\|", "|");
        }
        tr.append(cell);
      }
    }
    table.addEventListener("mousedown", event => {
      if (event.button !== 0) return;
      event.preventDefault();
      view.dispatch({ selection: { anchor: view.posAtDOM(table) } });
      view.focus();
    });
    return table;
  }
}

class MermaidWidget extends WidgetType {
  constructor(private readonly source: string, private readonly render: RenderDiagram) { super(); }

  eq(other: MermaidWidget) { return this.source === other.source; }

  toDOM(view: EditorView): HTMLElement {
    const node = view.dom.ownerDocument.createElement("pre");
    node.className = "mermaid";
    node.textContent = this.source;
    this.render(node, this.source);
    return node;
  }
}

function decorations(state: EditorState, render: RenderDiagram) {
  const ranges: Range<Decoration>[] = [];
  // Для спайка достаточно блоков верхнего уровня. Вложенные конструкции остаются исходником.
  const cursor = syntaxTree(state).cursor();
  if (!cursor.firstChild()) return Decoration.none;
  do {
    if (cursor.name === "Table") {
      const node = cursor.node;
      const from = state.doc.lineAt(node.from).from;
      if (state.selection.ranges.some(range => range.from <= node.to && range.to >= from)) continue;
      const widget = new TableWidget(state.sliceDoc(node.from, node.to), node);
      ranges.push(Decoration.replace({ block: true, widget }).range(from, node.to));
    } else if (cursor.name === "FencedCode") {
      const node = cursor.node;
      const info = node.getChild("CodeInfo");
      const closing = node.lastChild;
      if (!info || closing?.name !== "CodeMark" || closing.from <= info.to ||
          !/^mermaid(?:[\t ]|$)/i.test(state.sliceDoc(info.from, info.to))) continue;
      const code = node.getChild("CodeText");
      if (!code || code.to - code.from > maxDiagramLength) continue;
      const source = state.sliceDoc(code.from, code.to);
      if (!source.trim()) continue;
      ranges.push(Decoration.widget({ block: true, side: 1, widget: new MermaidWidget(source, render) }).range(node.to));
    }
  } while (cursor.nextSibling());
  return Decoration.set(ranges);
}

export function blockWidgets(render: RenderDiagram) {
  return StateField.define({
    create: state => decorations(state, render),
    update: (value, tr) => tr.docChanged || tr.selection || syntaxTree(tr.startState) !== syntaxTree(tr.state)
      ? decorations(tr.state, render)
      : value,
    provide: field => EditorView.decorations.from(field),
  });
}
