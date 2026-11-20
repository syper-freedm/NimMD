import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { clearMocks, mockConvertFileSrc, mockIPC } from "@tauri-apps/api/mocks";
import { showView } from "./view";

const root = document.createElement("div");

beforeEach(() => {
  document.body.append(root);
  vi.stubGlobal("matchMedia", () => ({ matches: false }));
});

afterEach(() => {
  root.replaceChildren();
  root.remove();
  for (const asset of document.head.querySelectorAll(
    '[src^="/vendor/"], [href^="/vendor/"]',
  ))
    asset.remove();
  vi.unstubAllGlobals();
  clearMocks();
});

function viewPayload(body: string) {
  return { body, windowLabel: "nimmd-window-3" };
}

function mockView(body: string): void {
  mockIPC((command) => {
    if (command === "get_view")
      return viewPayload('<main class="nimmd">' + body + "</main>");
    if (command === "get_launch")
      return { mode: "view", path: "note.md", errors: [] };
    throw new Error(command);
  });
}

async function loadAsset(path: string, event = "load"): Promise<void> {
  const asset = await vi.waitFor(() => {
    const node = document.head.querySelector(
      `[src="${path}"], [href="${path}"]`,
    );
    if (!node) throw new Error(`Ресурс не запрошен: ${path}`);
    return node;
  });
  asset.dispatchEvent(new Event(event));
}

test("загрузка и TXT используют IPC окна и импортируют текст без активных узлов", async () => {
  const commands: [string, unknown][] = [];
  let deliver!: (view: ReturnType<typeof viewPayload>) => void;
  const fragment = new Promise<ReturnType<typeof viewPayload>>((resolve) => {
    deliver = resolve;
  });
  mockIPC((command, args) => {
    commands.push([command, args]);
    return command === "get_view"
      ? fragment
      : { mode: "view", path: "note.txt", errors: [] };
  });
  const ready = showView(root);
  expect(root.querySelector('[role="status"]')?.textContent).toContain(
    "Загрузка",
  );
  deliver(
    viewPayload(
      '<main class="nimmd nimmd-data" data-kind="text"><pre class="data">&lt;b&gt;буквы&lt;/b&gt;</pre></main>',
    ),
  );
  await ready;
  expect(commands).toContainEqual(["get_view", {}]);
  expect(root.querySelector("main[data-kind=text] pre")?.textContent).toBe(
    "<b>буквы</b>",
  );
  expect(root.querySelector("b, [role=status]")).toBeNull();
  expect(
    document.head.querySelector('[src^="/vendor/"], [href^="/vendor/"]'),
  ).toBeNull();
});

test("картинки запрашиваются по схеме окна с сохранением исходного кодирования пути", async () => {
  mockConvertFileSrc("macos");
  mockView(
    '<img src="./assets/%D1%84%20a%23.png" alt="Схема"><img src="%2e%2e/photo.png" alt="Снаружи">',
  );
  await showView(root);
  const [image, denied] = root.querySelectorAll("img");
  expect(image?.getAttribute("src")).toBe(
    "nimmd-asset://localhost/nimmd-window-3?path=.%2Fassets%2F%25D1%2584%2520a%2523.png",
  );
  expect(denied?.getAttribute("src")).toBe(
    "nimmd-asset://localhost/nimmd-window-3?path=%252e%252e%2Fphoto.png",
  );
  denied?.dispatchEvent(new Event("error"));
  expect(root.querySelectorAll("img")).toHaveLength(1);
  expect(root.querySelector('[role="status"]')?.textContent).toContain(
    "Снаружи",
  );
});

test("внешние ссылки передаются системе, якоря работают, остальные переходы подавляются", async () => {
  const opened: unknown[] = [];
  mockIPC((command, args) => {
    if (command === "get_view")
      return viewPayload(
        '<main class="nimmd">' +
          '<a href="https://example.com/">Сайт</a><a href="mailto:example@example.com">Почта</a>' +
          '<a href="#note">Якорь</a><a href="file:///note.md">Файл</a><a href="">Нет адреса</a></main>',
      );
    if (command === "get_launch")
      return { mode: "view", path: "note.md", errors: [] };
    if (command === "open_external") {
      opened.push(args);
      return;
    }
    throw new Error(command);
  });
  await showView(root);
  const links = [...root.querySelectorAll("a")];
  const navigates = links.map((link) =>
    link.dispatchEvent(
      new MouseEvent("click", { bubbles: true, cancelable: true }),
    ),
  );
  expect(navigates).toEqual([false, false, true, false, false]);
  expect(opened).toEqual([
    { url: "https://example.com/" },
    { url: "mailto:example@example.com" },
  ]);
});

test("ошибка чтения показывается текстом с именем файла и способом повторить открытие", async () => {
  const error =
    "Не удалось прочитать «<img>.md». Закройте окно и откройте файл снова.";
  mockIPC((command) =>
    command === "get_view"
      ? Promise.reject(new Error(error))
      : { mode: "view", path: "note.md", errors: [] },
  );
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
      return viewPayload('<main class="nimmd"><p>Заметка</p></main>');
    }
    return { mode: "view", path: "note.md", errors };
  });
  await showView(root);
  const content = root.querySelector("main");
  expect(root.querySelector('[role="alert"]')?.textContent).toBe(error);
  expect(root.querySelector("img")).toBeNull();

  errors = [];
  root.dispatchEvent(new Event("nimmd-launch-changed"));
  await vi.waitFor(() => {
    expect(root.querySelector('[role="alert"]')).toBeNull();
  });
  expect(root.querySelector("main")).toBe(content);
  expect(reads).toBe(1);
});

test("KaTeX загружается отдельно и сохраняет исходник ошибочной формулы", async () => {
  mockView(
    '<span data-math-style="display">x+y</span><span data-math-style="inline">\\bad</span>',
  );
  const render = vi.fn((tex: string, node: HTMLElement) => {
    const result = document.createElement("span");
    result.className = tex === "x+y" ? "katex" : "katex-error";
    node.append(result);
  });
  vi.stubGlobal("katex", { render });

  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/katex.min.js"),
  ]);
  await ready;
  expect(
    root.querySelector('[data-math-style="display"] .katex'),
  ).not.toBeNull();
  expect(render.mock.calls[0]?.[0]).toBe("x+y");
  expect(render).toHaveBeenCalledWith(
    "x+y",
    expect.any(HTMLElement),
    expect.objectContaining({
      displayMode: true,
      trust: false,
      throwOnError: false,
    }),
  );
  expect(root.querySelector('[data-math-style="inline"]')?.textContent).toBe(
    "\\bad",
  );
  expect(root.querySelector('[role="status"]')?.textContent).toContain(
    "Не удалось отрисовать формулу",
  );
  expect(
    document.head.querySelector('[src="/vendor/mermaid.min.js"]'),
  ).toBeNull();
});

test("Mermaid получает оригинал и сохраняет исходник сбойного блока", async () => {
  const elk = elkDiagram;
  mockView(`<pre class="mermaid">${elk}</pre><pre class="mermaid">сбой</pre>`);
  const initialize = vi.fn();
  const diagramSources: string[] = [];
  const run = vi.fn(({ nodes }: { nodes: HTMLElement[] }) => {
    for (const node of nodes) {
      diagramSources.push(node.textContent);
      if (node.textContent === "сбой") continue;
      node.replaceChildren(
        document.createElementNS("http://www.w3.org/2000/svg", "svg"),
      );
    }
    return Promise.resolve();
  });
  vi.stubGlobal("mermaid", { initialize, run });

  const ready = showView(root);
  await loadAsset("/vendor/mermaid.min.js");
  await ready;
  expect(initialize).toHaveBeenCalledWith(
    expect.objectContaining({
      securityLevel: "strict",
      startOnLoad: false,
      layout: "dagre",
      theme: "default",
    }),
  );
  expect(diagramSources).toEqual([elk, "сбой"]);
  expect(root.querySelector("pre.mermaid > svg")).not.toBeNull();
  expect(root.querySelectorAll("pre.mermaid")[1]?.textContent).toBe("сбой");
  expect(root.querySelector('[role="status"]')?.textContent).toContain(
    "Не удалось отрисовать диаграмму",
  );
  expect(
    document.head.querySelector('[src="/vendor/katex.min.js"]'),
  ).toBeNull();
});

test("лимиты UTF-16 считаются совместно и пропуски не расходуют остаток", async () => {
  const math = (s: string) => `<span data-math-style="inline">${s}</span>`;
  const diagram = (s: string) => `<pre class="mermaid">${s}</pre>`;
  const maxMath = "😀".repeat(2048);
  const maxDiagram = "d".repeat(8192);
  mockView(
    math(maxMath + "x") +
      diagram(maxDiagram + "x") +
      diagram(maxDiagram) +
      math(maxMath).repeat(5) +
      math("a".repeat(4095)) +
      math("лишняя") +
      math("x") +
      diagram("лишняя диаграмма"),
  );
  const render = vi.fn();
  const inputs = captureDiagrams();
  vi.stubGlobal("katex", { render });

  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.js"),
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/mermaid.min.js"),
  ]);
  await ready;
  expect(inputs).toEqual([maxDiagram]);
  expect(render).toHaveBeenCalledTimes(7);
  expect(render).toHaveBeenLastCalledWith(
    "x",
    expect.any(HTMLElement),
    expect.any(Object),
  );
  expect(root.querySelector("pre.mermaid")?.textContent).toBe(maxDiagram + "x");
  expect(root.textContent).toContain("слишком длинная");
  expect(root.textContent).toContain("лимит документа");
});

const elkDiagram =
  '---\nconfig:\n  layout: elk\n---\nflowchart TD\nA["layout: elk"]';

test("Mermaid получает неизменённый текст на границе после пропущенных диаграмм", async () => {
  const prefix = elkDiagram + "\n%% ";
  const max = prefix + "x".repeat(8192 - prefix.length);
  const over = max + "x";
  const near = prefix + "x".repeat(8162 - prefix.length);
  const next = 'flowchart TD\nB["следующая"]';
  const diagram = (source: string) => `<pre class="mermaid">${source}</pre>`;
  mockView(
    diagram(over).repeat(4) +
      diagram(max) +
      diagram(near).repeat(3) +
      diagram(next),
  );
  const inputs = captureDiagrams();

  const ready = showView(root);
  await loadAsset("/vendor/mermaid.min.js");
  await ready;

  expect(inputs).toEqual([max, near, near, near, next]);
  expect(root.querySelector("pre.mermaid")?.textContent).toBe(over);
  expect(root.querySelectorAll("pre.mermaid > svg")).toHaveLength(5);
});

function captureDiagrams(): string[] {
  const inputs: string[] = [];
  vi.stubGlobal("mermaid", {
    initialize: vi.fn(),
    run: ({ nodes }: { nodes: HTMLElement[] }) => {
      for (const node of nodes) {
        inputs.push(node.textContent);
        node.replaceChildren(
          document.createElementNS("http://www.w3.org/2000/svg", "svg"),
        );
      }
      return Promise.resolve();
    },
  });
  return inputs;
}

test.each([0, 1])(
  "общий предел учитывает оригинал в порядке документа при добавке %i",
  async (extra) => {
    const math = (s: string) => `<span data-math-style="inline">${s}</span>`;
    const maxMath = "x".repeat(4096);
    const tail = "x".repeat(4096 - elkDiagram.length + extra);
    mockView(
      math(maxMath).repeat(7) +
        math(tail) +
        `<pre class="mermaid">${elkDiagram}</pre>` +
        math("y"),
    );
    const inputs = captureDiagrams();
    const renderedMath: string[] = [];
    vi.stubGlobal("katex", {
      render: (source: string, node: HTMLElement) => {
        renderedMath.push(source);
        const output = document.createElement("span");
        output.className = "katex";
        node.append(output);
      },
    });

    const ready = showView(root);
    await Promise.all([
      loadAsset("/vendor/katex.min.js"),
      loadAsset("/vendor/katex.min.css"),
      extra ? Promise.resolve() : loadAsset("/vendor/mermaid.min.js"),
    ]);
    await ready;

    const diagram = root.querySelector("pre.mermaid");
    const lastFormula = root.querySelectorAll("[data-math-style]")[8];
    if (extra) {
      expect(inputs).toEqual([]);
      expect(
        document.head.querySelector('[src="/vendor/mermaid.min.js"]'),
      ).toBeNull();
      expect(diagram?.textContent).toBe(elkDiagram);
      expect(diagram?.nextElementSibling?.textContent).toContain(
        "лимит документа",
      );
      expect(renderedMath.at(-1)).toBe("y");
      expect(lastFormula?.querySelector(".katex")).not.toBeNull();
    } else {
      expect(inputs).toEqual([elkDiagram]);
      expect(diagram?.querySelector("svg")).not.toBeNull();
      expect(lastFormula?.textContent).toBe("y");
      expect(lastFormula?.nextElementSibling?.textContent).toContain(
        "лимит документа",
      );
      expect(renderedMath).toHaveLength(8);
    }
  },
);

test("при сбое доставки одной библиотеки другая сохраняет возможность отрисовки", async () => {
  mockView(
    '<pre class="mermaid">graph TD\na--&gt;b</pre><span data-math-style="inline">x</span><p>Заметка</p>',
  );
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
  expect(root.querySelector("pre.mermaid")?.textContent).toBe(
    "graph TD\na-->b",
  );
  expect(root.querySelector(".katex")).not.toBeNull();
  expect(root.querySelector('[role="status"]')?.textContent).toContain(
    "Ресурс недоступен",
  );
  expect(root.textContent).toContain("Заметка");
});

test("предел количества блоков общий для формул и диаграмм", async () => {
  mockView(
    '<span data-math-style="inline">x</span>'.repeat(31) +
      '<pre class="mermaid">graph TD</pre><span data-math-style="inline">y</span>',
  );
  const render = vi.fn();
  const inputs = captureDiagrams();
  vi.stubGlobal("katex", { render });

  const ready = showView(root);
  await Promise.all([
    loadAsset("/vendor/katex.min.js"),
    loadAsset("/vendor/katex.min.css"),
    loadAsset("/vendor/mermaid.min.js"),
  ]);
  await ready;
  expect(render).toHaveBeenCalledTimes(31);
  expect(inputs).toEqual(["graph TD"]);
  expect(root.querySelectorAll("[data-math-style]")[31]?.textContent).toBe("y");
  expect(root.textContent).toContain("Превышен лимит документа");
});
