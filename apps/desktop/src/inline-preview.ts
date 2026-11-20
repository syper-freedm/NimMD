import { syntaxTree } from "@codemirror/language";
import type { EditorState, Range } from "@codemirror/state";
import {
  Decoration,
  WidgetType,
  EditorView,
  type DecorationSet,
} from "@codemirror/view";

import { invoke } from "@tauri-apps/api/core";
import { isExternalLink } from "./document-fragment";
import { decodeMarkdown } from "./markdown";

const hidden = Decoration.replace({});
const inlineTags: Readonly<Record<string, readonly [string, string]>> = {
  StrongEmphasis: ["strong", "EmphasisMark"],
  Emphasis: ["em", "EmphasisMark"],
  Strikethrough: ["s", "StrikethroughMark"],
  InlineCode: ["code", "CodeMark"],
};
const alertTitles: Readonly<Record<string, string>> = {
  note: "Note",
  tip: "Tip",
  important: "Important",
  warning: "Warning",
  caution: "Caution",
};

export function selectionTouches(
  state: EditorState,
  from: number,
  to: number,
): boolean {
  const first = state.doc.lineAt(from).from;
  const last = state.doc.lineAt(Math.max(from, to - 1)).to;
  return state.selection.ranges.some(
    (selection) => selection.from <= last && selection.to >= first,
  );
}

class TaskCheckbox extends WidgetType {
  constructor(
    readonly from: number,
    readonly checked: boolean,
  ) {
    super();
  }

  eq(other: TaskCheckbox): boolean {
    return this.from === other.from && this.checked === other.checked;
  }

  toDOM(view: EditorView): HTMLInputElement {
    const checkbox = view.dom.ownerDocument.createElement("input");
    checkbox.type = "checkbox";
    checkbox.className = "task-list-item-checkbox";
    checkbox.checked = this.checked;
    checkbox.setAttribute("aria-label", "Задача выполнена");
    checkbox.addEventListener("change", () => {
      const current = view.state.sliceDoc(this.from, this.from + 3);
      if (view.state.readOnly || !/^\[[ xX]\]$/.test(current)) {
        checkbox.checked = this.checked;
        return;
      }
      view.dispatch({
        changes: {
          from: this.from + 1,
          to: this.from + 2,
          insert: checkbox.checked ? "x" : " ",
        },
        userEvent: "input",
      });
      // После замены виджета возвращаем клавиатурный ввод в редактор.
      view.focus();
    });
    return checkbox;
  }
}

class TextWidget extends WidgetType {
  constructor(
    readonly text: string,
    readonly className: string,
  ) {
    super();
  }

  eq(other: TextWidget): boolean {
    return this.text === other.text && this.className === other.className;
  }

  toDOM(view: EditorView): HTMLElement {
    const span = view.dom.ownerDocument.createElement("span");
    span.textContent = this.text;
    span.className = this.className;
    return span;
  }
}

function clickedLink(event: MouseEvent): string | undefined {
  const url =
    event.target instanceof Element
      ? event.target.closest("a[href]")?.getAttribute("href")
      : null;
  return url && isExternalLink(url) ? url : undefined;
}

function openLink(event: MouseEvent, view: EditorView): boolean {
  const url = clickedLink(event);
  if (!url) return false;
  event.preventDefault();
  void invoke("open_external", { url }).catch(() => {
    view.dispatch({
      effects: EditorView.announce.of("Не удалось открыть ссылку."),
    });
  });
  return true;
}

export const editorLinks = EditorView.domEventHandlers({
  mousedown(event) {
    if (event.button > 1 || !clickedLink(event)) return false;
    // Иначе курсор перейдёт в строку ссылки и скроет её до события click.
    event.preventDefault();
    return true;
  },
  click: openLink,
  auxclick: (event, view) => event.button === 1 && openLink(event, view),
});

type MarkdownTree = ReturnType<typeof syntaxTree>;
const linkReferences = new WeakMap<MarkdownTree, Map<string, string>>();

function referenceUrl(state: EditorState, label: string): string {
  const tree = syntaxTree(state);
  const key = (text: string) => text.trim().replace(/\s+/g, " ").toLowerCase();
  let definitions = linkReferences.get(tree);
  if (!definitions) {
    const found = new Map<string, string>();
    tree.iterate({
      enter({ name, node }) {
        if (name !== "LinkReference") return;
        const label = node.getChild("LinkLabel");
        const url = node.getChild("URL");
        if (!label || !url) return;
        const normalized = key(state.sliceDoc(label.from + 1, label.to - 1));
        if (!found.has(normalized))
          found.set(normalized, state.sliceDoc(url.from, url.to));
      },
    });
    definitions = found;
    linkReferences.set(tree, definitions);
  }
  return definitions.get(key(label)) ?? "";
}

function linkDecorations(
  state: EditorState,
  node: MarkdownTree["topNode"],
): Range<Decoration>[] {
  const { name, from, to } = node;
  const typed = name === "URL";
  const url = typed ? node : node.getChild("URL");
  const [open, close] = node.getChildren("LinkMark");
  if (
    typed &&
    ["Link", "Autolink", "Image", "LinkReference"].includes(
      node.parent?.name ?? "",
    )
  )
    return [];
  const textFrom = open?.to ?? from;
  const textTo = close?.from ?? to;
  if (textFrom >= textTo) return [];
  let address = "";
  if (url) {
    address = state.sliceDoc(url.from, url.to);
  } else if (name === "Link") {
    const label = node.getChild("LinkLabel");
    address = referenceUrl(
      state,
      (label && state.sliceDoc(label.from + 1, label.to - 1)) ||
        state.sliceDoc(textFrom, textTo),
    );
  }
  address = decodeMarkdown(address.replace(/^<([\s\S]*)>$/, "$1"));
  if (typed || name === "Autolink") {
    if (/^www\./i.test(address)) address = `http://${address}`;
    else if (/^[^:\s@]+@[^\s@]+$/.test(address)) address = `mailto:${address}`;
  }
  if (!isExternalLink(address)) return [];
  const ranges = [
    Decoration.mark({ tagName: "a", attributes: { href: address } }).range(
      textFrom,
      textTo,
    ),
  ];
  if (from < textFrom) ranges.push(hidden.range(from, textFrom));
  if (textTo < to) ranges.push(hidden.range(textTo, to));
  return ranges;
}

export function inlineDecorations(view: EditorView): DecorationSet {
  const { state, visibleRanges } = view;
  const ranges: Range<Decoration>[] = [];
  const seenLines = new Map<number, Set<string>>();
  const addLines = (
    start: number,
    end: number,
    visibleFrom: number,
    visibleTo: number,
    className: string,
  ) => {
    const last = state.doc.lineAt(
      Math.min(Math.max(start, end - 1), visibleTo),
    ).number;
    for (
      let number = state.doc.lineAt(Math.max(start, visibleFrom)).number;
      number <= last;
      number++
    ) {
      const { from, to } = state.doc.line(number);
      if (selectionTouches(state, from, to)) continue;
      const classes = seenLines.get(from) ?? new Set<string>();
      if (!classes.has(className)) {
        classes.add(className);
        seenLines.set(from, classes);
        ranges.push(Decoration.line({ class: className }).range(from));
      }
    }
  };

  for (const { from, to } of visibleRanges) {
    syntaxTree(state).iterate({
      from,
      to,
      enter({ name, from: start, to: end, node }) {
        if (
          name === "Alert" ||
          name === "Blockquote" ||
          name === "FootnoteDefinition"
        ) {
          if (name === "Alert") {
            const marker = node.getChild("AlertMarker");
            const type =
              marker &&
              state.sliceDoc(marker.from + 2, marker.to - 1).toLowerCase();
            if (type && Object.hasOwn(alertTitles, type)) {
              addLines(
                start,
                end,
                from,
                to,
                `markdown-alert markdown-alert-${type}`,
              );
            }
          } else {
            addLines(
              start,
              end,
              from,
              to,
              name === "Blockquote" ? "cm-quote" : "footnotes",
            );
          }
        }

        if (start < from || end > to || selectionTouches(state, start, end))
          return;
        const tag = inlineTags[name];
        const level = /^(?:ATX|Setext)Heading([1-6])$/.exec(name)?.[1];
        if (tag) {
          ranges.push(Decoration.mark({ tagName: tag[0] }).range(start, end));
          for (const marker of node.getChildren(tag[1]))
            ranges.push(hidden.range(marker.from, marker.to));
        } else if (level) {
          addLines(start, end, from, to, `cm-heading cm-heading-${level}`);
          for (const marker of node.getChildren("HeaderMark")) {
            const line = state.doc.lineAt(marker.from);
            const first = state.sliceDoc(line.from, marker.from).trim() === "";
            const space =
              first && /^[ \t]$/.test(state.sliceDoc(marker.to, marker.to + 1));
            ranges.push(hidden.range(marker.from, marker.to + Number(space)));
          }
        } else if (
          name === "ListMark" &&
          node.parent?.parent?.name === "BulletList"
        ) {
          if (node.parent.getChild("Task")) {
            ranges.push(hidden.range(start, end));
          } else {
            ranges.push(
              Decoration.replace({
                widget: new TextWidget("•", "cm-bullet"),
              }).range(start, end),
            );
          }
        } else if (name === "TaskMarker") {
          addLines(start, end, from, to, "contains-task-list task-list-item");
          ranges.push(
            Decoration.replace({
              widget: new TaskCheckbox(
                start,
                state.sliceDoc(start + 1, end - 1).toLowerCase() === "x",
              ),
            }).range(start, end),
          );
        } else if (name === "QuoteMark") {
          const space = /^[ \t]$/.test(state.sliceDoc(end, end + 1));
          ranges.push(hidden.range(start, end + Number(space)));
        } else if (name === "Link" || name === "Autolink" || name === "URL") {
          for (const range of linkDecorations(state, node)) ranges.push(range);
        } else if (name === "AlertMarker") {
          const type = state.sliceDoc(start + 2, end - 1).toLowerCase();
          const title = Object.hasOwn(alertTitles, type)
            ? alertTitles[type]
            : undefined;
          if (title) {
            ranges.push(
              Decoration.replace({
                widget: new TextWidget(title, "markdown-alert-title"),
              }).range(start, end),
            );
          }
        } else if (
          name === "WikiLink" ||
          name === "FootnoteReference" ||
          name === "FootnoteMark"
        ) {
          const className =
            name === "WikiLink" ? "cm-wikilink" : "footnote-ref";
          ranges.push(Decoration.mark({ class: className }).range(start, end));
        }
      },
    });
  }
  return Decoration.set(ranges, true);
}
