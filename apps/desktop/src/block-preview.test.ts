import { clearMocks, mockConvertFileSrc } from "@tauri-apps/api/mocks";
import type { EditorView } from "@codemirror/view";
import { afterEach, expect, test, vi } from "vitest";
import { mountEditor } from "./editor";

let editor: EditorView | undefined;

function open(source: string, windowLabel?: string): EditorView {
  editor = mountEditor(document.body, source, { windowLabel });
  return editor;
}

afterEach(() => {
  editor?.destroy();
  editor = undefined;
  document.body.replaceChildren();
  document.head.querySelector('[src="/vendor/mermaid.min.js"]')?.remove();
  clearMocks();
  vi.unstubAllGlobals();
});

test("клик по формуле показывает исходник, а уход курсора снова её сворачивает", () => {
  const source = "Первая\n\nТекст $x+y$ рядом.";
  const view = open(source);
  const formula = view.contentDOM.querySelector("[data-math-style]");
  expect(formula?.textContent).toBe("x+y");

  formula?.dispatchEvent(
    new MouseEvent("mousedown", { bubbles: true, cancelable: true }),
  );
  expect(view.contentDOM.querySelector("[data-math-style]")).toBeNull();
  expect(view.contentDOM.textContent).toContain("$x+y$");

  view.dispatch({ selection: { anchor: 0 } });
  expect(view.contentDOM.querySelector("[data-math-style]")?.textContent).toBe(
    "x+y",
  );
  expect(view.state.doc.toString()).toBe(source);
});

test("Mermaid из цитаты получает исходные переносы без префиксов цитаты", async () => {
  const rendered: string[] = [];
  const run = vi.fn(({ nodes }: { nodes: HTMLElement[] }) => {
    for (const node of nodes) {
      rendered.push(node.textContent);
      node.replaceChildren(
        document.createElementNS("http://www.w3.org/2000/svg", "svg"),
      );
    }
    return Promise.resolve();
  });
  vi.stubGlobal("mermaid", { initialize: vi.fn(), run });
  vi.stubGlobal("matchMedia", () => ({
    matches: false,
    addListener() {},
    removeListener() {},
  }));

  const source = [
    "Первая",
    "",
    "> ```mermaid",
    "> flowchart TD",
    ">",
    "> A --> B",
    "> ```",
  ].join("\n");
  const view = open(source);
  const script = await vi.waitFor(() => {
    const asset = document.head.querySelector('[src="/vendor/mermaid.min.js"]');
    expect(asset).not.toBeNull();
    return asset;
  });
  script?.dispatchEvent(new Event("load"));
  await vi.waitFor(() => {
    expect(view.contentDOM.querySelector(".mermaid svg")).not.toBeNull();
  });
  expect(run).toHaveBeenCalledOnce();
  expect(rendered).toEqual(["flowchart TD\n\nA --> B"]);
  expect(view.state.doc.toString()).toBe(source);
});

test("разрешает ссылочные картинки, нормализует URL и не запрашивает внешние схемы", () => {
  mockConvertFileSrc("macos");
  const source = [
    "Первая",
    "",
    "![Путь][IMAGE REF]",
    "",
    "![Снаружи][remote]",
    "",
    "[image ref]: <folder/name\\(a\\)&amp;b.png>",
    "[remote]: h&#x74;tps://example.com/image.png",
  ].join("\n");
  const view = open(source, "nimmd-window-1");
  const images = view.contentDOM.querySelectorAll("img[alt]");
  expect(images).toHaveLength(1);
  expect(images[0]?.getAttribute("src")).toBe(
    "nimmd-asset://localhost/nimmd-window-1?path=folder%2Fname%28a%29%26b.png",
  );
  expect(view.contentDOM.querySelector(".remote-image")?.textContent).toBe(
    "https://example.com/image.png",
  );

  images[0]?.dispatchEvent(new Event("error"));
  expect(view.contentDOM.textContent).toContain("Картинка недоступна: Путь");
  expect(view.state.doc.toString()).toBe(source);
});

test("оставляет доступ к исходнику таблицы, которая превышает бюджет DOM", () => {
  const columns = 72;
  const source = [
    "Первая",
    "",
    `| ${Array.from({ length: columns }, (_, index) => `h${index.toString()}`).join(" | ")} |`,
    `| ${Array.from({ length: columns }, () => "---").join(" | ")} |`,
    ...Array.from({ length: 64 }, () => "| один |"),
  ].join("\n");
  const view = open(source);
  const expanded = view.contentDOM.querySelector("button");
  expect(view.contentDOM.querySelector("table")).toBeNull();
  expect(expanded?.textContent).toBe("Показать исходник большой таблицы");
  expanded?.click();
  expect(view.contentDOM.textContent).toContain("| h0 | h1 | h2 |");
  expect(view.state.doc.toString()).toBe(source);
});
