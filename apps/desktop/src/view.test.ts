import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { showView } from "./view";

const root = document.createElement("div");

beforeEach(() => {
  document.body.append(root);
  vi.stubGlobal("matchMedia", () => ({ matches: false }));
});

afterEach(() => {
  root.replaceChildren();
  root.remove();
  for (const asset of document.head.querySelectorAll('[src^="/vendor/"], [href^="/vendor/"]')) asset.remove();
  vi.unstubAllGlobals();
  clearMocks();
});

function mockView(body: string): void {
  mockIPC((command) => {
    if (command === "get_view") return '<main class="nimmd">' + body + "</main>";
    if (command === "get_launch") return { mode: "view", path: "note.md", errors: [] };
    throw new Error(command);
  });
}

async function loadAsset(path: string, event = "load"): Promise<void> {
  const asset = await vi.waitFor(() => {
    const node = document.head.querySelector(`[src="${path}"], [href="${path}"]`);
    if (!node) throw new Error(`Ресурс не запрошен: ${path}`);
    return node;
  });
  asset.dispatchEvent(new Event(event));
}

test("загрузка и TXT используют IPC окна и импортируют текст без активных узлов", async () => {
  const commands: [string, unknown][] = [];
  let deliver!: (body: string) => void;
  const fragment = new Promise<string>((resolve) => { deliver = resolve; });
  mockIPC((command, args) => {
    commands.push([command, args]);
    return command === "get_view"
      ? fragment
      : { mode: "view", path: "note.txt", errors: [] };
  });
  const ready = showView(root);
  expect(root.querySelector('[role="status"]')?.textContent).toContain("Загрузка");
  deliver('<main class="nimmd nimmd-data" data-kind="text"><pre class="data">&lt;b&gt;буквы&lt;/b&gt;</pre></main>');
  await ready;
  expect(commands).toContainEqual(["get_view", {}]);
  expect(root.querySelector("main[data-kind=text] pre")?.textContent).toBe("<b>буквы</b>");
  expect(root.querySelector("b, [role=status]")).toBeNull();
  expect(document.head.querySelector('[src^="/vendor/"], [href^="/vendor/"]')).toBeNull();
});

test("ошибка чтения показывается текстом с именем файла и способом повторить открытие", async () => {
  const error = "Не удалось прочитать «<img>.md». Закройте окно и откройте файл снова.";
  mockIPC((command) => command === "get_view"
    ? Promise.reject(new Error(error))
    : { mode: "view", path: "note.md", errors: [] });
  await showView(root);
  expect(root.querySelector('[role="alert"]')?.textContent).toBe(error);
  expect(root.querySelector("img")).toBeNull();
  expect(root.querySelector('[role="status"]')).toBeNull();
});

test("оконная ошибка обновляется отдельно от открытого документа", async () => {
  const error = "Не удалось открыть «<img>.md».";
  let errors = [error];
  let reads = 0;
  mockIPC((command) => {
    if (command === "get_view") {
      reads += 1;
      return '<main class="nimmd"><p>Заметка</p></main>';
    }
    return { mode: "view", path: "note.md", errors };
  });
  await showView(root);
  const content = root.querySelector("main");
  expect(root.querySelector('[role="alert"]')?.textContent).toBe(error);
  expect(root.querySelector("img")).toBeNull();

  errors = [];
  root.dispatchEvent(new Event("nimmd-launch-changed"));
  await vi.waitFor(() => { expect(root.querySelector('[role="alert"]')).toBeNull(); });
  expect(root.querySelector("main")).toBe(content);
  expect(reads).toBe(1);
});

test("KaTeX загружается отдельно и сохраняет исходник ошибочной формулы", async () => {
  mockView('<span data-math-style="display">x+y</span><span data-math-style="inline">\\bad</span>');
  const render = vi.fn((tex: string, node: HTMLElement) => {
    const result = document.createElement("span");
    result.className = tex === "x+y" ? "katex" : "katex-error";
    node.append(result);
  });
  vi.stubGlobal("katex", { render });

  const ready = showView(root);
  await Promise.all([loadAsset("/vendor/katex.min.css"), loadAsset("/vendor/katex.min.js")]);
  await ready;
  expect(root.querySelector('[data-math-style="display"] .katex')).not.toBeNull();
  expect(render.mock.calls[0]?.[0]).toBe("x+y");
  expect(render).toHaveBeenCalledWith("x+y", expect.any(HTMLElement), expect.objectContaining({
    displayMode: true, trust: false, throwOnError: false,
  }));
  expect(root.querySelector('[data-math-style="inline"]')?.textContent).toBe("\\bad");
  expect(root.querySelector('[role="status"]')?.textContent).toContain("Не удалось отрисовать формулу");
  expect(document.head.querySelector('[src="/vendor/mermaid.min.js"]')).toBeNull();
});

test("Mermaid меняет только выбор ELK, сохраняя текст подписи и исходник сбойного блока", async () => {
  const elk = '---\nconfig:\n  layout: elk\n---\nflowchart TD\nA["layout: elk"]';
  mockView(`<pre class="mermaid">${elk}</pre><pre class="mermaid">сбой</pre>`);
  const initialize = vi.fn();
  const parse = vi.fn((text: string) => Promise.resolve({ config: text === elk ? { layout: "elk" } : {} }));
  const diagramSources: string[] = [];
  const run = vi.fn(({ nodes }: { nodes: HTMLElement[] }) => {
    for (const node of nodes) {
      diagramSources.push(node.textContent);
      if (node.textContent === "сбой") continue;
      node.replaceChildren(document.createElementNS("http://www.w3.org/2000/svg", "svg"));
    }
    return Promise.resolve();
  });
  vi.stubGlobal("mermaid", { initialize, parse, run });

  const ready = showView(root);
  await loadAsset("/vendor/mermaid.min.js");
  await ready;
  expect(initialize).toHaveBeenCalledWith(expect.objectContaining({
    securityLevel: "strict", startOnLoad: false, layout: "dagre", theme: "default",
  }));
  expect(parse.mock.calls[0]?.[0]).toBe(elk);
  expect(diagramSources[0]).toBe(elk + '\n%%{init: {"layout":"dagre"}}%%');
  expect(root.querySelector("pre.mermaid > svg")).not.toBeNull();
  expect(root.querySelectorAll("pre.mermaid")[1]?.textContent).toBe("сбой");
  expect(root.querySelector('[role="status"]')?.textContent).toContain("Не удалось отрисовать диаграмму");
  expect(document.head.querySelector('[src="/vendor/katex.min.js"]')).toBeNull();
});

test("лимиты UTF-16 считаются совместно и пропуски не расходуют остаток", async () => {
  const math = (s: string) => `<span data-math-style="inline">${s}</span>`;
  const diagram = (s: string) => `<pre class="mermaid">${s}</pre>`;
  const maxMath = "😀".repeat(2048);
  const maxDiagram = "d".repeat(8192);
  mockView(
    math(maxMath + "x") + diagram(maxDiagram + "x") + diagram(maxDiagram) +
    math(maxMath).repeat(5) + math("a".repeat(4095)) + math("лишняя") +
    math("x") + diagram("лишняя диаграмма"),
  );
  const render = vi.fn();
  const parse = vi.fn(() => Promise.resolve({ config: {} }));
  vi.stubGlobal("katex", { render });
  vi.stubGlobal("mermaid", { parse, initialize: vi.fn(), run: () => Promise.resolve() });

  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.js"),
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/mermaid.min.js"),
  ]);
  await ready;
  expect(parse).toHaveBeenCalledTimes(1);
  expect(render).toHaveBeenCalledTimes(7);
  expect(render).toHaveBeenLastCalledWith("x", expect.any(HTMLElement), expect.any(Object));
  expect(root.querySelector('pre.mermaid')?.textContent).toBe(maxDiagram + "x");
  expect(root.textContent).toContain("слишком длинная");
  expect(root.textContent).toContain("лимит документа");
});

test("при сбое доставки одной библиотеки другая сохраняет возможность отрисовки", async () => {
  mockView('<pre class="mermaid">graph TD\na--&gt;b</pre><span data-math-style="inline">x</span><p>Заметка</p>');
  const render = vi.fn((_tex: string, node: HTMLElement) => {
    const result = document.createElement("span");
    result.className = "katex";
    node.append(result);
  });
  vi.stubGlobal("katex", { render });
  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.js"),
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/mermaid.min.js", "error"),
  ]);
  await ready;
  expect(root.querySelector("pre.mermaid")?.textContent).toBe("graph TD\na-->b");
  expect(root.querySelector(".katex")).not.toBeNull();
  expect(root.querySelector('[role="status"]')?.textContent).toContain("Ресурс недоступен");
  expect(root.textContent).toContain("Заметка");
});

test("предел количества блоков общий для формул и диаграмм", async () => {
  mockView('<span data-math-style="inline">x</span>'.repeat(31) +
    '<pre class="mermaid">graph TD</pre><span data-math-style="inline">y</span>');
  const render = vi.fn();
  const parse = vi.fn(() => Promise.resolve({ config: {} }));
  vi.stubGlobal("katex", { render });
  vi.stubGlobal("mermaid", { parse, initialize: vi.fn(), run: () => Promise.resolve() });

  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.js"),
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/mermaid.min.js"),
  ]);
  await ready;
  expect(render).toHaveBeenCalledTimes(31);
  expect(parse).toHaveBeenCalledOnce();
  expect(root.querySelectorAll("[data-math-style]")[31]?.textContent).toBe("y");
  expect(root.textContent).toContain("Превышен лимит документа");
});
