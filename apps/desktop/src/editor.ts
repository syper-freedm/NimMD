import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { syntaxTree } from "@codemirror/language";
import { EditorState, StateEffect, StateField, Text, type Range } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, keymap, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { markdownLanguage } from "./markdown";

const togglePreview = StateEffect.define();
const previewEnabled = StateField.define({
  create: () => true,
  update(enabled, transaction) {
    return transaction.effects.some((effect) => effect.is(togglePreview)) ? !enabled : enabled;
  },
});

const strongText = Decoration.mark({ tagName: "strong" });
const hiddenMarker = Decoration.replace({});

function previewDecorations(view: EditorView): DecorationSet {
  const { state, visibleRanges } = view;
  if (!state.field(previewEnabled)) return Decoration.none;

  const ranges: Range<Decoration>[] = [];
  for (const { from, to } of visibleRanges) {
    syntaxTree(state).iterate({
      from,
      to,
      enter({ name, from: start, to: end }) {
        if (name !== "StrongEmphasis") return;
        const line = state.doc.lineAt(start);
        if (end > line.to || start < from || end > to
          || state.selection.ranges.some((selection) => selection.from <= line.to && selection.to >= line.from)) return;
        ranges.push(hiddenMarker.range(start, start + 2), strongText.range(start + 2, end - 2),
          hiddenMarker.range(end - 2, end));
      },
    });
  }
  return Decoration.set(ranges, true);
}

const livePreview = ViewPlugin.fromClass(class {
  decorations: DecorationSet;

  constructor(view: EditorView) {
    this.decorations = previewDecorations(view);
  }

  update(update: ViewUpdate): void {
    if (update.docChanged || update.selectionSet || update.viewportChanged
      || syntaxTree(update.state) !== syntaxTree(update.startState)
      || update.state.field(previewEnabled) !== update.startState.field(previewEnabled)) {
      this.decorations = previewDecorations(update.view);
    }
  }
}, { decorations: (plugin) => plugin.decorations });

const editorTheme = EditorView.theme({
  "&": { minHeight: "12rem" },
  ".cm-content": { fontFamily: "inherit", padding: "1rem 0" },
  ".cm-line": { padding: "0" },
});

function normalizeCrlf(source: string): string {
  return source.replaceAll("\r\n", "\n");
}

export function mountEditor(parent: HTMLElement, source: string): EditorView {
  const doc = Text.of(normalizeCrlf(source).split("\n"));
  return new EditorView({
    parent,
    state: EditorState.create({
      doc,
      extensions: [
        EditorState.lineSeparator.of("\n"),
        EditorView.clipboardInputFilter.of(normalizeCrlf),
        EditorView.contentAttributes.of({ "aria-label": "Исходный текст Markdown" }),
        EditorView.lineWrapping,
        markdownLanguage(doc),
        history(),
        previewEnabled,
        livePreview,
        editorTheme,
        keymap.of([
          { key: "Mod-/", run(view) {
            view.dispatch({ effects: togglePreview.of(null) });
            return true;
          } },
          ...defaultKeymap.filter((binding) => binding.key !== "Mod-/"),
          ...historyKeymap,
        ]),
      ],
    }),
  });
}
