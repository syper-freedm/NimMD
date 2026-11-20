import type { syntaxTree } from "@codemirror/language";
import { WidgetType, type EditorView } from "@codemirror/view";

const maxCells = 4096;

type MarkdownNode = ReturnType<typeof syntaxTree>["topNode"];

export class TableWidget extends WidgetType {
  constructor(
    private readonly source: string,
    private readonly block: MarkdownNode,
  ) {
    super();
  }

  eq(other: TableWidget) {
    return this.source === other.source;
  }

  toDOM(view: EditorView): HTMLElement {
    const document = view.dom.ownerDocument;
    const delimiter = this.block.getChild("TableDelimiter");
    const alignments = delimiter
      ? this.source
          .slice(
            delimiter.from - this.block.from,
            delimiter.to - this.block.from,
          )
          .trim()
          .replace(/^\||\|$/g, "")
          .split("|")
          .map((value) => {
            const marker = value.trim();
            return marker.endsWith(":")
              ? marker.startsWith(":")
                ? "center"
                : "right"
              : marker.startsWith(":")
                ? "left"
                : "";
          })
      : [];
    const select = (element: HTMLElement) => {
      view.dispatch({ selection: { anchor: view.posAtDOM(element) } });
      view.focus();
    };
    const columns = alignments.length;
    const rows: MarkdownNode[] = [];
    for (let row = this.block.firstChild; row; row = row.nextSibling) {
      if (row.name !== "TableHeader" && row.name !== "TableRow") continue;
      if ((rows.length + 1) * columns > maxCells) {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = "Показать исходник большой таблицы";
        button.addEventListener("click", () => {
          select(button);
        });
        return button;
      }
      rows.push(row);
    }
    const table = document.createElement("table");
    const head = table.createTHead();
    const body = table.createTBody();
    for (const row of rows) {
      const header = row.name === "TableHeader";
      const tr = (header ? head : body).insertRow();
      const from = row.from - this.block.from;
      const to = row.to - this.block.from;
      const pipes = row
        .getChildren("TableDelimiter")
        .map((pipe) => pipe.from - this.block.from);
      if (pipes[0] !== from) pipes.unshift(from - 1);
      if (
        pipes.at(-1) !==
        from + this.source.slice(from, to).trimEnd().length - 1
      )
        pipes.push(to);
      for (let column = 0; column < columns; column++) {
        const cell = view.dom.ownerDocument.createElement(header ? "th" : "td");
        const alignment = alignments[column];
        if (alignment) cell.style.textAlign = alignment;
        const left = pipes[column];
        const right = pipes[column + 1];
        if (left !== undefined && right !== undefined) {
          cell.textContent = this.source
            .slice(left + 1, right)
            .trim()
            .replaceAll("\\|", "|");
        }
        tr.append(cell);
      }
    }
    table.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      select(table);
    });
    return table;
  }
}
