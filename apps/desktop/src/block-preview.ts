import { syntaxTree } from "@codemirror/language";
import { StateField, type EditorState, type Range } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";
import { documentImageUrl } from "./document-fragment";
import {
  maxBlocks,
  maxDiagramLength,
  maxMathLength,
  maxTotalLength,
  renderDiagrams,
  renderMath,
} from "./render-blocks";
import { decodeMarkdown } from "./markdown";
import { TableWidget } from "./table-widget";

class FrontmatterWidget extends WidgetType {
  toDOM(view: EditorView): HTMLElement {
    const button = view.dom.ownerDocument.createElement("button");
    button.type = "button";
    button.className = "frontmatter";
    button.textContent = "Frontmatter · YAML";
    button.setAttribute("aria-expanded", "false");
    button.addEventListener("click", () => {
      view.dispatch({ selection: { anchor: view.state.doc.line(2).from } });
      view.focus();
    });
    return button;
  }
}

class ImageWidget extends WidgetType {
  constructor(
    readonly source: string,
    readonly alt: string,
    readonly windowLabel: string | undefined,
  ) {
    super();
  }

  eq(other: ImageWidget): boolean {
    return (
      this.source === other.source &&
      this.alt === other.alt &&
      this.windowLabel === other.windowLabel
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const box = view.dom.ownerDocument.createElement("div");
    box.className = "cm-image";
    const source = decodeMarkdown(this.source);
    const alt = decodeMarkdown(this.alt);
    if (/^https?:/i.test(source)) {
      box.classList.add("remote-image");
      box.textContent = source;
    } else if (
      this.windowLabel &&
      source &&
      !/^(?:[\w+.-]+:|[/\\])/.test(source)
    ) {
      const image = view.dom.ownerDocument.createElement("img");
      image.alt = alt;
      image.addEventListener(
        "error",
        () => {
          box.textContent = alt
            ? `Картинка недоступна: ${alt}`
            : "Картинка недоступна.";
        },
        { once: true },
      );
      image.src = documentImageUrl(this.windowLabel, source);
      box.append(image);
    } else {
      box.textContent = alt || "Картинка недоступна.";
    }
    return box;
  }
}

class FormulaWidget extends WidgetType {
  constructor(
    readonly source: string,
    readonly display: boolean,
  ) {
    super();
  }

  eq(other: FormulaWidget): boolean {
    return this.source === other.source && this.display === other.display;
  }

  toDOM(view: EditorView): HTMLElement {
    const tag = this.display ? "div" : "span";
    const container: HTMLElement = view.dom.ownerDocument.createElement(tag);
    const node = container.appendChild(
      view.dom.ownerDocument.createElement(tag),
    );
    container.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      view.dispatch({ selection: { anchor: view.posAtDOM(container) } });
      view.focus();
    });
    node.dataset.mathStyle = this.display ? "display" : "inline";
    node.textContent = this.source;
    requestAnimationFrame(() => {
      if (node.isConnected) void renderMath([{ node, source: this.source }]);
    });
    return container;
  }
}

class DiagramWidget extends WidgetType {
  constructor(readonly source: string) {
    super();
  }

  eq(other: DiagramWidget): boolean {
    return this.source === other.source;
  }

  toDOM(view: EditorView): HTMLElement {
    const container = view.dom.ownerDocument.createElement("div");
    const node = container.appendChild(
      view.dom.ownerDocument.createElement("pre"),
    );
    node.className = "mermaid";
    node.textContent = this.source;
    requestAnimationFrame(() => {
      if (node.isConnected)
        void renderDiagrams([{ node, source: this.source }]);
    });
    return container;
  }
}

interface BlockDecoration {
  readonly from: number;
  readonly to: number;
  readonly decoration: Range<Decoration>;
}

interface PendingImage {
  readonly from: number;
  readonly to: number;
  readonly alt: string;
  readonly url: string | null;
  readonly label: string;
}

type CodePart = { from: number; to: number };

function diagramSource(state: EditorState, parts: readonly CodePart[]): string {
  let result = "";
  let previousLine = -1;
  for (const { from, to } of parts) {
    const line = state.doc.lineAt(from).number;
    if (previousLine >= 0) result += "\n".repeat(line - previousLine);
    result += state.sliceDoc(from, to);
    previousLine = state.doc.lineAt(to).number;
  }
  return result;
}

const blockNames = new Set([
  "Frontmatter",
  "Table",
  "Image",
  "InlineMath",
  "DisplayMath",
  "FencedCode",
]);

function collectBlocks(
  state: EditorState,
  windowLabel: string | undefined,
): BlockDecoration[] {
  const blocks: BlockDecoration[] = [];
  const images: PendingImage[] = [];
  const definitions = new Map<string, string>();
  const reference = (label: string) =>
    label.trim().replace(/\s+/g, " ").toLowerCase();
  let dynamicBlocks = 0;
  let dynamicLength = 0;
  const allow = (length: number, max: number) => {
    if (
      length > max ||
      dynamicBlocks >= maxBlocks ||
      dynamicLength + length > maxTotalLength
    )
      return false;
    dynamicBlocks++;
    dynamicLength += length;
    return true;
  };

  syntaxTree(state).iterate({
    enter(cursor) {
      const { name } = cursor;
      if (!blockNames.has(name) && name !== "LinkReference") return;
      const { from, to, node } = cursor;
      if (name === "LinkReference") {
        const label = node.getChild("LinkLabel");
        const url = node.getChild("URL");
        if (label && url) {
          const key = reference(state.sliceDoc(label.from + 1, label.to - 1));
          if (!definitions.has(key))
            definitions.set(key, state.sliceDoc(url.from, url.to));
        }
        return false;
      }

      const first = state.doc.lineAt(from);
      // Конец узла не входит в его диапазон. У frontmatter он после последнего перевода строки.
      const last = state.doc.lineAt(Math.max(from, to - 1));
      const activeFrom = name === "Frontmatter" ? first.from + 1 : first.from;
      const add = (decoration: Range<Decoration>) => {
        blocks.push({ from: activeFrom, to: last.to, decoration });
      };
      if (name === "Frontmatter" || name === "Table") {
        const widget =
          name === "Frontmatter"
            ? new FrontmatterWidget()
            : new TableWidget(state.sliceDoc(from, to), node);
        add(
          Decoration.replace({ block: true, widget }).range(
            first.from,
            Math.min(last.to, to),
          ),
        );
        return false;
      }
      if (name === "Image") {
        const url = node.getChild("URL");
        const close = node.getChildren("LinkMark")[1];
        const label = node.getChild("LinkLabel");
        const alt = close ? state.sliceDoc(from + 2, close.from) : "";
        images.push({
          from: first.from,
          to: last.to,
          alt,
          url: url && state.sliceDoc(url.from, url.to),
          label: reference(
            (label && state.sliceDoc(label.from + 1, label.to - 1)) || alt,
          ),
        });
        return false;
      }
      if (name === "InlineMath" || name === "DisplayMath") {
        const display = name === "DisplayMath";
        const delimiter = display ? 2 : 1;
        if (allow(to - from - 2 * delimiter, maxMathLength)) {
          const widget = new FormulaWidget(
            state.sliceDoc(from + delimiter, to - delimiter),
            display,
          );
          if (
            display &&
            !state.sliceDoc(first.from, from).trim() &&
            !state.sliceDoc(to, last.to).trim()
          ) {
            add(
              Decoration.replace({ block: true, widget }).range(
                first.from,
                last.to,
              ),
            );
          } else if (first.number === last.number) {
            add(Decoration.replace({ widget }).range(from, to));
          } else {
            add(
              Decoration.widget({ block: true, side: 1, widget }).range(
                last.to,
              ),
            );
          }
        }
        return false;
      }

      const info = node.getChild("CodeInfo");
      if (
        info &&
        /^mermaid(?:[\t ]|$)/i.test(state.sliceDoc(info.from, info.to)) &&
        node.getChildren("CodeMark").length === 2
      ) {
        const code = node.getChildren("CodeText");
        const start = code[0];
        const end = code.at(-1);
        if (start && end && end.to - start.from <= maxDiagramLength) {
          const source = diagramSource(state, code);
          if (source.trim() && allow(source.length, maxDiagramLength)) {
            add(
              Decoration.widget({
                block: true,
                side: 1,
                widget: new DiagramWidget(source),
              }).range(last.to),
            );
          }
        }
      }
      return false;
    },
  });
  for (const { from, to, alt, label, url } of images) {
    const raw = url ?? definitions.get(label);
    if (!raw) continue;
    const source =
      raw.startsWith("<") && raw.endsWith(">") ? raw.slice(1, -1) : raw;
    const widget = new ImageWidget(source, alt, windowLabel);
    blocks.push({
      from,
      to,
      decoration: Decoration.widget({ block: true, side: 1, widget }).range(to),
    });
  }
  return blocks;
}

function visibleDecorations(
  state: EditorState,
  blocks: readonly BlockDecoration[],
): DecorationSet {
  const selection = state.selection.ranges;
  const decorations: Range<Decoration>[] = [];
  for (const { from, to, decoration } of blocks) {
    if (!selection.some((range) => range.from <= to && range.to >= from)) {
      decorations.push(decoration);
    }
  }
  return Decoration.set(decorations, true);
}

export function blockPreview(
  windowLabel: string | undefined,
  enabled: (state: EditorState) => boolean,
) {
  interface Preview {
    readonly blocks: readonly BlockDecoration[];
    readonly decorations: DecorationSet;
  }
  const empty: Preview = { blocks: [], decorations: Decoration.none };
  const build = (state: EditorState): Preview => {
    const blocks = collectBlocks(state, windowLabel);
    return { blocks, decorations: visibleDecorations(state, blocks) };
  };
  return StateField.define({
    create: (state) => (enabled(state) ? build(state) : empty),
    update(value, transaction) {
      if (!enabled(transaction.state)) return empty;
      if (
        !enabled(transaction.startState) ||
        transaction.docChanged ||
        syntaxTree(transaction.startState) !== syntaxTree(transaction.state)
      ) {
        return build(transaction.state);
      }
      return transaction.selection
        ? {
            blocks: value.blocks,
            decorations: visibleDecorations(transaction.state, value.blocks),
          }
        : value;
    },
    provide: (field) =>
      EditorView.decorations.from(field, (value) => value.decorations),
  });
}
