import { execFileSync } from "node:child_process";

import { ensureSyntaxTree, type syntaxTree } from "@codemirror/language";
import { clearMocks, mockConvertFileSrc } from "@tauri-apps/api/mocks";
import { afterEach, beforeAll, expect, test, vi } from "vitest";
import contract from "../../../packages/theme/CONTRACT.md?raw";
import { mountEditor } from "./editor";

const collapsedCallout = {
  name: "сворачиваемый callout",
  markdown: "> [!note]-",
  active: "> [!note]-",
  preview: ".cm-quote",
};
const fixtures = [
  {
    name: "заголовок",
    markdown: "# Заголовок 🦀",
    active: "# Заголовок 🦀",
    preview: ".cm-heading-1",
  },
  {
    name: "задача выполнена",
    markdown: "- [x] готово",
    active: "- [x] готово",
    preview: ".task-list-item-checkbox",
  },
  {
    name: "задача не выполнена",
    markdown: "- [ ] ещё",
    active: "- [ ] ещё",
    preview: ".task-list-item-checkbox",
  },
  {
    name: "вложенные списки",
    markdown: "3. Порядок\n   - Подпункт\n4. Продолжение",
    active: "   - Подпункт",
    preview: ".cm-bullet",
  },
  {
    name: "таблица",
    markdown: "| одно | два | три |\n| :-- | :-: | --: |\n| А | Б | В |",
    active: "| А | Б | В |",
    preview: "table",
  },
  ...["NOTE", "tip", "IMPORTANT", "WARNING", "CAUTION"].map((kind) => ({
    name: `алерт ${kind}`,
    markdown: `> [!${kind}]\n> Текст`,
    active: `> [!${kind}]`,
    preview: ".markdown-alert-title",
  })),
  {
    name: "callout Obsidian",
    markdown: "> [!abstract]",
    active: "> [!abstract]",
    preview: ".cm-quote",
  },
  collapsedCallout,
  {
    name: "сноска",
    markdown: "Сноска[^1]\n\n[^1]: пояснение",
    active: "[^1]: пояснение",
    preview: ".footnotes",
  },
  {
    name: "картинка",
    markdown: "![Схема](./img.png)",
    active: "![Схема](./img.png)",
    preview: ".cm-image img",
  },
  {
    name: "формула внутри строки",
    markdown: "Формула $x^2$.",
    active: "Формула $x^2$.",
    preview: '[data-math-style="inline"]',
  },
  {
    name: "формула отдельным блоком",
    markdown: "$$\nx^2\n$$",
    active: "x^2",
    preview: '[data-math-style="display"]',
  },
  {
    name: "Mermaid",
    markdown: "```mermaid\ngraph TD\n  A --> B\n```",
    active: "```mermaid",
    preview: ".mermaid",
  },
  {
    name: "frontmatter",
    markdown: "---\ntitle: Проверка\n---",
    active: "title: Проверка",
    preview: ".frontmatter",
  },
  { name: "блок кода", markdown: "```rust\nlet crab = '🦀';\n\n// конец\n```" },
].map((fixture) => ({
  ...fixture,
  source: `${fixture.markdown}\n\nПоследняя строка\n`,
}));

interface CoreDocument {
  body: string;
  tree: unknown[];
}

const sharedClasses = [
  ...(
    contract.split("## Общие с редактором\n")[1]?.split("\n## ")[0] ?? ""
  ).matchAll(/^- `([\w-]+)`$/gm),
].map((match) => match[1] ?? "");

let core: CoreDocument[];

beforeAll(() => {
  core = JSON.parse(
    execFileSync(
      "cargo",
      [
        "run",
        "--quiet",
        "--locked",
        "-p",
        "nimmd-core",
        "--example",
        "editor_contract",
      ],
      {
        input: JSON.stringify(fixtures.map(({ source }) => source)),
        encoding: "utf8",
        timeout: 120_000,
      },
    ),
  ) as CoreDocument[];
  expect(core).toHaveLength(fixtures.length);
}, 125_000);

afterEach(() => {
  clearMocks();
  vi.unstubAllGlobals();
});

type SyntaxNode = ReturnType<typeof syntaxTree>["topNode"];

function lezerTree(root: SyntaxNode, source: string): unknown[] {
  const text = (node: SyntaxNode | null) =>
    node ? source.slice(node.from, node.to) : "";
  const project = (node: SyntaxNode): unknown[] => {
    const children = () => {
      const result: unknown[] = [];
      for (let child = node.firstChild; child; child = child.nextSibling)
        result.push(...project(child));
      return result;
    };
    const heading = /^(?:ATX|Setext)Heading([1-6])$/.exec(node.name);
    if (heading) {
      const marks = node.getChildren("HeaderMark");
      const content = marks.reduceRight(
        (original, mark) =>
          original.slice(0, mark.from - node.from) +
          original.slice(mark.to - node.from),
        text(node),
      );
      return [
        { kind: "heading", level: Number(heading[1]), text: content.trim() },
      ];
    }
    switch (node.name) {
      case "BulletList":
      case "OrderedList": {
        const ordered = node.name === "OrderedList";
        const mark = node.getChild("ListItem")?.getChild("ListMark") ?? null;
        return [
          {
            kind: "list",
            ordered,
            start: ordered ? Number(text(mark).slice(0, -1)) : 0,
            children: children(),
          },
        ];
      }
      case "ListItem": {
        const task = node.getChild("Task");
        const marker = task?.getChild("TaskMarker");
        return [
          {
            kind: "item",
            checked: marker ? /^\[[xX]\]$/.test(text(marker)) : null,
            text:
              marker && task
                ? source.slice(marker.to, task.to).trim()
                : text(node.getChild("Paragraph")),
            children: children(),
          },
        ];
      }
      case "Table": {
        const delimiter = text(node.getChild("TableDelimiter"))
          .trim()
          .replace(/^\||\|$/g, "");
        const alignment = delimiter.split("|").map((cell) => {
          const left = cell.trim().startsWith(":");
          const right = cell.trim().endsWith(":");
          return left ? (right ? "center" : "left") : right ? "right" : "none";
        });
        return [{ kind: "table", alignment, children: children() }];
      }
      case "TableHeader":
      case "TableRow":
        return [
          {
            kind: "row",
            header: node.name === "TableHeader",
            children: children(),
          },
        ];
      case "TableCell":
        return [{ kind: "cell", text: text(node).replaceAll("\\|", "|") }];
      case "FencedCode":
      case "CodeBlock": {
        const parts = node.getChildren("CodeText");
        const first = parts[0];
        const last = parts.at(-1);
        return [
          {
            kind: "code",
            language:
              text(node.getChild("CodeInfo")).trim().split(/\s+/, 1)[0] ?? "",
            text: first && last ? source.slice(first.from, last.to) : "",
          },
        ];
      }
      case "Alert":
        return [
          {
            kind: "alert",
            type: text(node.getChild("AlertMarker")).slice(2, -1).toLowerCase(),
            text: text(node.getChild("Paragraph")),
            children: children(),
          },
        ];
      case "Blockquote":
        return [
          {
            kind: "quote",
            text: text(node.getChild("Paragraph")),
            children: children(),
          },
        ];
      case "Image":
        return [{ kind: "image", url: text(node.getChild("URL")) }];
      case "InlineMath":
      case "DisplayMath": {
        const display = node.name === "DisplayMath";
        const delimiter = display ? 2 : 1;
        return [
          {
            kind: "math",
            display,
            text: text(node).slice(delimiter, -delimiter).trim(),
          },
        ];
      }
      case "Document":
      case "Paragraph":
        return children();
      // Исключения из структурной проекции:
      // - Frontmatter отделяется до вызова comrak и сверяется по DOM.
      // - Сноски comrak перемещает в конец и удаляет неиспользуемые; сверяем их классы.
      // - Неподдерживаемые узлы и лексические маркеры Lezer не входят в общий JSON.
      default:
        return [];
    }
  };
  return project(root);
}

function classesIn(element: ParentNode): string[] {
  return sharedClasses.filter((name) => element.querySelector(`.${name}`));
}

test.each(
  fixtures.map((fixture, index) => [fixture.name, fixture, index] as const),
)(
  "%s совпадает с ядром и сохраняет исходник при смене позиции курсора",
  (_name, fixture, index) => {
    mockConvertFileSrc("macos");
    // У виджетов здесь проверяется DOM-обвязка. Рендер библиотек проверяется отдельно.
    vi.stubGlobal("requestAnimationFrame", () => 1);
    const parent = document.body.appendChild(document.createElement("main"));
    const editor = mountEditor(parent, fixture.source, {
      windowLabel: "nimmd-window-contract",
    });
    try {
      editor.dispatch({
        selection: { anchor: fixture.source.indexOf("Последняя строка") },
      });
      const snapshot = core[index];
      expect(snapshot).toBeDefined();
      if (!snapshot) throw new Error("Нет результата рендера ядра");
      const html = new DOMParser().parseFromString(snapshot.body, "text/html");
      const shared = classesIn(editor.contentDOM);
      const coreClasses = classesIn(html);

      const tree = ensureSyntaxTree(editor.state, fixture.source.length, 1000);
      expect(tree).not.toBeNull();
      if (!tree) throw new Error("Lezer не разобрал документ");
      const projected = lezerTree(tree.topNode, fixture.source);
      if (fixture.name === collapsedCallout.name) {
        // comrak принимает '-' за заголовок NOTE; редактор сохраняет callout как цитату.
        expect(coreClasses).toEqual([
          "markdown-alert",
          "markdown-alert-title",
          "markdown-alert-note",
        ]);
        expect(shared).toEqual([]);
        expect(snapshot.tree).toEqual([
          { kind: "alert", type: "note", text: "", children: [] },
        ]);
        expect(projected).toEqual([
          { kind: "quote", text: "[!note]-", children: [] },
        ]);
      } else {
        expect(shared).toEqual(coreClasses);
        expect(projected).toEqual(snapshot.tree);
      }

      if ("preview" in fixture) {
        expect(editor.contentDOM.querySelector(fixture.preview)).not.toBeNull();
        if (fixture.preview === ".cm-quote")
          expect(editor.contentDOM.querySelector(".markdown-alert")).toBeNull();
        expect(editor.state.doc.toString()).toBe(fixture.source);
        editor.dispatch({
          selection: { anchor: fixture.source.indexOf(fixture.active) },
        });
        expect(editor.contentDOM.querySelector(fixture.preview)).toBeNull();
        expect(
          [...editor.contentDOM.querySelectorAll(".cm-line")].map(
            (line) => line.textContent,
          ),
        ).toContain(fixture.active);
      }
      expect(editor.state.doc.toString()).toBe(fixture.source);
    } finally {
      editor.destroy();
      parent.remove();
    }
  },
);

test("общий корпус проверяет каждый класс контракта", () => {
  expect(sharedClasses.length).toBeGreaterThan(0);
  const seen = new Set(
    core.flatMap(({ body }) =>
      classesIn(new DOMParser().parseFromString(body, "text/html")),
    ),
  );
  expect([...seen].sort()).toEqual([...sharedClasses].sort());
});
