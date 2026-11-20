import { syntaxTree } from "@codemirror/language";
import { StateEffect, type EditorState, type Range } from "@codemirror/state";
import {
  Decoration,
  ViewPlugin,
  type EditorView,
  type ViewUpdate,
} from "@codemirror/view";
import { invoke } from "@tauri-apps/api/core";

interface Span {
  start: number;
  end: number;
  class: string;
}

interface Piece {
  from: number;
  to: number;
  offset: number;
}

const repaint = StateEffect.define();
const limit = 8 * 1024;
const encoder = new TextEncoder();
type Node = ReturnType<typeof syntaxTree>["topNode"];

function* codeParts(node: Node) {
  for (let part = node.getChild("CodeText"); part; part = part.nextSibling) {
    if (part.name === "CodeText") yield part;
  }
}

function codeBlock(state: EditorState, node: Node) {
  if (node.name === "Frontmatter") {
    const from = state.doc.lineAt(node.from).to + 1;
    const to = state.doc.lineAt(Math.max(from, node.to - 1)).from;
    return { lang: "yaml", parts: [{ from, to }] };
  }
  const info = node.getChild("CodeInfo");
  const lang = info
    ? (state.sliceDoc(info.from, info.to).trim().split(/\s+/, 1)[0] ?? "")
    : "";
  return { lang, parts: codeParts(node) };
}

export function codePreview(enabled: (state: EditorState) => boolean) {
  return ViewPlugin.fromClass(
    class {
      decorations: ReturnType<typeof Decoration.set>;
      private readonly cache = new Map<string, { spans?: Span[] }>();
      private destroyed = false;

      constructor(private readonly view: EditorView) {
        this.decorations = this.build();
      }

      update(update: ViewUpdate): void {
        if (
          update.docChanged ||
          update.viewportChanged ||
          enabled(update.state) !== enabled(update.startState) ||
          syntaxTree(update.state) !== syntaxTree(update.startState) ||
          update.transactions.some((transaction) =>
            transaction.effects.some((effect) => effect.is(repaint)),
          )
        ) {
          this.decorations = this.build();
        }
      }

      destroy(): void {
        this.destroyed = true;
      }

      private build() {
        const { state, viewport } = this.view;
        const used = new Set<string>();
        const ranges: Range<Decoration>[] = [];
        let budget = limit;
        function buildCode(parts: Iterable<{ from: number; to: number }>) {
          const pieces: Piece[] = [];
          let code = "";
          let previousLine = -1;
          for (const { from, to } of parts) {
            const line = state.doc.lineAt(from).number;
            const gap = previousLine < 0 ? 0 : line - previousLine;
            const remaining = budget - code.length - gap;
            if (remaining <= 0) break;
            const end = Math.min(to, from + remaining);
            code += "\n".repeat(gap);
            pieces.push({ from, to: end, offset: code.length });
            code += state.sliceDoc(from, end);
            previousLine = state.doc.lineAt(end).number;
            if (end < to) break;
          }
          if (code && !code.endsWith("\n")) code += "\n";
          // В Rust ограничение измеряется в байтах, а позиции редактора — в UTF-16.
          if (encoder.encode(code).length > budget) {
            const buffer = new Uint8Array(budget);
            code = code.slice(0, encoder.encodeInto(code, buffer).read);
          }
          return { code, pieces };
        }

        const addCode = (
          lang: string,
          parts: Iterable<{ from: number; to: number }>,
        ): number => {
          const { code, pieces } = buildCode(parts);
          if (!code) return 0;
          const key = `${lang}\0${code}`;
          used.add(key);
          const spans = this.load(key, lang, code);
          let first = 0;
          for (const { from, to, offset } of pieces) {
            const end = Math.min(offset + to - from, code.length);
            while (first < spans.length && (spans[first]?.end ?? 0) <= offset)
              first++;
            for (let index = first; index < spans.length; index++) {
              const span = spans[index];
              if (!span || span.start >= end) break;
              const start = Math.max(span.start, offset);
              const stop = Math.min(span.end, end);
              if (start < stop)
                ranges.push(
                  Decoration.mark({ class: span.class }).range(
                    from + start - offset,
                    from + stop - offset,
                  ),
                );
            }
          }
          return encoder.encode(code).length;
        };

        if (enabled(state)) {
          syntaxTree(state).iterate({
            ...viewport,
            enter({ name, from, to, node }) {
              if (
                name !== "FencedCode" &&
                name !== "CodeBlock" &&
                name !== "Frontmatter"
              )
                return;
              if (budget <= 0 || from > viewport.to || to < viewport.from)
                return false;
              const { lang, parts } = codeBlock(state, node);
              if (lang.toLowerCase() !== "mermaid")
                budget -= addCode(lang, parts);
              return false;
            },
          });
        }
        for (const key of this.cache.keys()) {
          if (!used.has(key)) this.cache.delete(key);
        }
        return Decoration.set(ranges, true);
      }

      private load(key: string, lang: string, code: string): Span[] {
        const cached = this.cache.get(key);
        if (cached) return cached.spans ?? [];
        const pending: { spans?: Span[] } = {};
        this.cache.set(key, pending);
        void Promise.resolve()
          .then(() => invoke<Span[]>("highlight", { lang, code }))
          .then((spans) => {
            if (this.destroyed || this.cache.get(key) !== pending) return;
            pending.spans = spans;
            this.view.dispatch({ effects: repaint.of(null) });
          })
          .catch(() => {});
        return [];
      }
    },
    { decorations: (plugin) => plugin.decorations },
  );
}
