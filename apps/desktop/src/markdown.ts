import { markdown } from "@codemirror/lang-markdown";
import { yamlFrontmatter } from "@codemirror/lang-yaml";
import {
  Compartment,
  EditorState,
  type Extension,
  type Text,
  type Transaction,
} from "@codemirror/state";
import { GFM, type InlineParser, type MarkdownConfig } from "@lezer/markdown";

function token(
  name: string,
  marker: string,
  pattern: RegExp,
  before: string,
): InlineParser {
  const markerCode = marker.charCodeAt(0);
  return {
    name,
    before,
    parse(context, next, position) {
      if (next !== markerCode) return -1;

      pattern.lastIndex = position - context.offset;
      const match = pattern.exec(context.text);
      return match
        ? context.addElement(
            context.elt(name, position, position + match[0].length),
          )
        : -1;
    },
  };
}

const nimmdMarkdown: MarkdownConfig = {
  defineNodes: [
    "InlineMath",
    "DisplayMath",
    "WikiLink",
    "FootnoteReference",
    "FootnoteMark",
    "AlertMarker",
    {
      name: "FootnoteDefinition",
      block: true,
      composite(_context, line) {
        const indent = line.baseIndent + 4;
        if (line.indent < indent) return line.pos === line.text.length;

        line.moveBaseColumn(indent);
        return true;
      },
    },
    {
      name: "Alert",
      block: true,
      composite(context, line) {
        if (line.text[line.pos] !== ">") return false;

        const mark = context.lineStart + line.pos;
        line.addMarker(context.elt("QuoteMark", mark, mark + 1));
        const next = line.text[line.pos + 1];
        line.moveBase(line.pos + (next === " " || next === "\t" ? 2 : 1));
        return true;
      },
    },
  ],
  parseInline: [
    token(
      "DisplayMath",
      "$",
      /(?<!\$)\$\$(?!\$)(?:\\[^\n]|[^\\$]|\$(?!\$))+?\$\$(?!\$)/y,
      "Emphasis",
    ),
    token(
      "InlineMath",
      "$",
      /(?<!\$)\$(?![\s$])(?:\\[^\n]|[^\\$])+?(?<!\s)\$(?![\d$])/y,
      "Emphasis",
    ),
    token("WikiLink", "[", /\[\[[^\][\n]+\]\]/y, "Link"),
    token("FootnoteReference", "[", /\[\^[^\s\][]+\](?!\()/y, "Link"),
  ],
  parseBlock: [
    {
      name: "Alert",
      before: "Blockquote",
      parse(context, line) {
        if (line.text[line.pos] !== ">") return false;

        const header =
          /^> \[!(?:NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?=[ \t]|$)/i.exec(
            line.text.slice(line.pos),
          )?.[0];
        if (!header) return false;

        const start = context.lineStart + line.pos;
        context.startComposite("Alert", line.pos);
        context.addElement(context.elt("QuoteMark", start, start + 1));
        context.addElement(
          context.elt(
            "AlertMarker",
            start + header.indexOf("["),
            start + header.length,
          ),
        );
        line.moveBase(line.text.length);
        return null;
      },
    },
    {
      name: "FootnoteDefinition",
      after: "IndentedCode",
      parse(context, line) {
        if (line.text[line.pos] !== "[") return false;

        const marker = /^\[\^[^\s\][]+\]:/.exec(line.text.slice(line.pos))?.[0];
        if (!marker) return false;

        const start = context.lineStart + line.pos;
        context.startComposite("FootnoteDefinition", line.pos);
        context.addElement(
          context.elt("FootnoteMark", start, start + marker.length),
        );
        line.moveBase(line.skipSpace(line.pos + marker.length));
        return null;
      },
    },
  ],
};

const markdownBody = markdown({
  extensions: [GFM, nimmdMarkdown],
  addKeymap: false,
});
const markdownYaml = yamlFrontmatter({ content: markdownBody });
const languageConfig = new Compartment();

function hasClosingDelimiter(doc: Text, first = 2, last = doc.lines): boolean {
  if (last < first) return false;

  for (const line of doc.iterLines(first, last + 1)) {
    if (line === "---") return true;
  }
  return false;
}

function languageFor(doc: Text) {
  // Ядро считает frontmatter только блоком между двумя полными строками ---.
  return doc.line(1).text === "---" && hasClosingDelimiter(doc)
    ? markdownYaml
    : markdownBody;
}

function changesTouchDelimiter(transaction: Transaction): boolean {
  const { doc } = transaction.startState;
  const newDoc = transaction.newDoc;
  const touchesClosing = (text: Text, from: number, to: number) =>
    hasClosingDelimiter(
      text,
      Math.max(2, text.lineAt(from).number),
      text.lineAt(to).number,
    );

  let touched = false;
  transaction.changes.iterChangedRanges((fromA, toA, fromB, toB) => {
    touched ||=
      touchesClosing(doc, fromA, toA) || touchesClosing(newDoc, fromB, toB);
  });
  return touched;
}

export function markdownLanguage(doc: Text): Extension {
  return [
    languageConfig.of(languageFor(doc)),
    EditorState.transactionExtender.of((transaction) => {
      if (!transaction.docChanged) return null;

      const doc = transaction.newDoc;
      const opensYaml = doc.line(1).text === "---";
      const openedYaml = transaction.startState.doc.line(1).text === "---";
      if (
        opensYaml === openedYaml &&
        (!opensYaml || !changesTouchDelimiter(transaction))
      ) {
        return null;
      }

      const support = languageFor(doc);
      return languageConfig.get(transaction.startState) === support
        ? null
        : { effects: languageConfig.reconfigure(support) };
    }),
  ];
}
