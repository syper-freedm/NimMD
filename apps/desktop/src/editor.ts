import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { syntaxTree } from "@codemirror/language";
import {
  Compartment,
  EditorState,
  StateEffect,
  StateField,
} from "@codemirror/state";
import {
  Decoration,
  EditorView,
  ViewPlugin,
  keymap,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { blockPreview } from "./block-preview";
import { codePreview } from "./code-preview";
import { editorText, normalizeCrlf } from "./editor-file";
import { editorLinks, inlineDecorations } from "./inline-preview";
import { markdownLanguage } from "./markdown";

const editability = new Compartment();

export function setEditorLocked(view: EditorView, locked: boolean): void {
  if (view.state.readOnly === locked) return;
  view.dispatch({
    effects: editability.reconfigure([
      EditorState.readOnly.of(locked),
      EditorView.editable.of(!locked),
    ]),
  });
}

const togglePreview = StateEffect.define();
const previewEnabled = StateField.define({
  create: () => true,
  update(enabled, transaction) {
    return transaction.effects.some((effect) => effect.is(togglePreview))
      ? !enabled
      : enabled;
  },
});

function previewDecorations(view: EditorView): DecorationSet {
  return view.state.field(previewEnabled)
    ? inlineDecorations(view)
    : Decoration.none;
}

const livePreview = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = previewDecorations(view);
    }

    update(update: ViewUpdate): void {
      if (
        update.docChanged ||
        update.selectionSet ||
        update.viewportChanged ||
        syntaxTree(update.state) !== syntaxTree(update.startState) ||
        update.state.field(previewEnabled) !==
          update.startState.field(previewEnabled)
      ) {
        this.decorations = previewDecorations(update.view);
      }
    }
  },
  { decorations: (plugin) => plugin.decorations },
);

const editorTheme = EditorView.theme({
  "&": { minHeight: "12rem" },
  ".cm-content": { fontFamily: "inherit", padding: "1rem 0" },
  ".cm-line": { padding: "0" },
});

export function mountEditor(
  parent: HTMLElement,
  source: string,
  {
    windowLabel,
    onChange,
    onSave,
  }: {
    windowLabel?: string;
    onChange?: () => void;
    onSave?: () => void;
  } = {},
): EditorView {
  const doc = editorText(source);
  return new EditorView({
    parent,
    state: EditorState.create({
      doc,
      extensions: [
        editability.of([]),
        EditorState.lineSeparator.of("\n"),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange?.();
        }),
        EditorView.clipboardInputFilter.of(normalizeCrlf),
        EditorView.contentAttributes.of({
          "aria-label": "Исходный текст Markdown",
        }),
        EditorView.lineWrapping,
        markdownLanguage(doc),
        history(),
        previewEnabled,
        blockPreview(windowLabel, (state) => state.field(previewEnabled)),
        codePreview((state) => state.field(previewEnabled)),
        livePreview,
        editorLinks,
        editorTheme,
        keymap.of([
          {
            key: "Mod-s",
            run() {
              if (!onSave) return false;
              onSave();
              return true;
            },
          },
          {
            key: "Mod-/",
            run(view) {
              view.dispatch({ effects: togglePreview.of(null) });
              return true;
            },
          },
          ...defaultKeymap.filter((binding) => binding.key !== "Mod-/"),
          ...historyKeymap,
        ]),
      ],
    }),
  });
}
