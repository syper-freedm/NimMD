import { invoke } from "@tauri-apps/api/core";
import { documentFragment, showBlockMessage, type RenderedView } from "./document-fragment";
import type { Launch } from "./launch";

type Block = { node: HTMLElement; source: string };

interface ViewLibraries extends Window {
  katex?: {
    render: (source: string, node: HTMLElement, options: {
      displayMode: boolean; trust: false; throwOnError: false; errorColor: string;
    }) => void;
  };
  mermaid?: {
    initialize: (options: {
      securityLevel: "strict"; startOnLoad: false; layout: "dagre";
      theme: "default" | "dark"; suppressErrorRendering: true; deterministicIds: true;
    }) => void;
    run: (options: { nodes: HTMLElement[]; suppressErrors: true }) => Promise<void>;
  };
}

const libraries: ViewLibraries = window;
const maxMathLength = 4096;
const maxDiagramLength = 8192;
const maxBlocks = 32;
const maxTotalLength = 32_768;
const documentLimit = "Превышен лимит документа для формул и диаграмм.";
const diagramTooLong = "Диаграмма слишком длинная.";
const viewError = "Не удалось показать документ. Закройте окно и откройте файл снова.";
const mathErrorColor = "var(--nimmd-katex-error)";

function loadAsset(asset: HTMLLinkElement | HTMLScriptElement): Promise<void> {
  return new Promise((resolve, reject) => {
    asset.addEventListener("load", () => { resolve(); }, { once: true });
    asset.addEventListener("error", () => {
      asset.remove();
      reject(new Error("Ресурс недоступен"));
    }, { once: true });
    document.head.append(asset);
  });
}

async function renderMath(blocks: Block[]): Promise<void> {
  if (!blocks.length) return;
  const css = document.createElement("link");
  css.rel = "stylesheet";
  css.href = "/vendor/katex.min.css";
  const script = document.createElement("script");
  script.src = "/vendor/katex.min.js";
  try {
    await Promise.all([loadAsset(css), loadAsset(script)]);
    if (!libraries.katex) throw new Error("KaTeX недоступен");
  } catch {
    for (const { node } of blocks) showBlockMessage(node, "Не удалось отрисовать формулу. Ресурс недоступен.");
    return;
  }

  for (const { node, source } of blocks) {
    const output = document.createElement("span");
    try {
      libraries.katex.render(source, output, {
        displayMode: node.dataset.mathStyle === "display",
        trust: false,
        throwOnError: false,
        errorColor: mathErrorColor,
      });
      // KaTeX с throwOnError: false помечает неизвестные команды цветом в MathML.
      if (output.querySelector(`.katex-error, [mathcolor="${mathErrorColor}"]`)) {
        throw new Error("Ошибка TeX");
      }
      node.replaceChildren(...output.childNodes);
    } catch {
      showBlockMessage(node, "Не удалось отрисовать формулу.");
    }
  }
}

async function renderDiagrams(diagrams: Block[]): Promise<void> {
  if (!diagrams.length) return;
  // Самозапуск Mermaid привязан к load. Подключаем скрипт после события.
  if (document.readyState !== "complete") {
    await new Promise<void>((resolve) => {
      window.addEventListener("load", () => { resolve(); }, { once: true });
    });
  }

  const script = document.createElement("script");
  script.src = "/vendor/mermaid.min.js";
  let mermaid;
  try {
    await loadAsset(script);
    mermaid = libraries.mermaid;
    if (!mermaid) throw new Error("Mermaid недоступен");
    mermaid.initialize({
      securityLevel: "strict",
      startOnLoad: false,
      layout: "dagre",
      theme: window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "default",
      suppressErrorRendering: true,
      deterministicIds: true,
    });
  } catch {
    for (const { node } of diagrams) showBlockMessage(node, "Не удалось отрисовать диаграмму. Ресурс недоступен.");
    return;
  }
  try {
    await mermaid.run({ nodes: diagrams.map(({ node }) => node), suppressErrors: true });
  } catch {
    // Результат и исходник проверяются для каждого блока отдельно.
  }
  for (const { node, source } of diagrams) {
    if (node.querySelector(":scope > svg")) continue;
    node.textContent = source;
    node.removeAttribute("data-processed");
    showBlockMessage(node, "Не удалось отрисовать диаграмму.");
  }
}

async function renderBlocks(main: HTMLElement): Promise<void> {
  const math: Block[] = [];
  const diagrams: Block[] = [];
  let total = 0;
  for (const node of main.querySelectorAll<HTMLElement>("[data-math-style], pre.mermaid")) {
    const source = node.textContent;
    const formula = node.hasAttribute("data-math-style");
    if (source.length > (formula ? maxMathLength : maxDiagramLength)) {
      showBlockMessage(node, formula ? "Формула слишком длинная." : diagramTooLong);
      continue;
    }
    if (math.length + diagrams.length >= maxBlocks || total + source.length > maxTotalLength) {
      showBlockMessage(node, documentLimit);
      continue;
    }
    (formula ? math : diagrams).push({ node, source });
    total += source.length;
  }
  await Promise.all([renderMath(math), renderDiagrams(diagrams)]);
}

function watchWindowErrors(root: HTMLElement, notice: HTMLElement): Promise<void> {
  let generation = 0;
  async function refresh(): Promise<void> {
    const request = ++generation;
    let errors: string[];
    try {
      const launch = await invoke<Launch>("get_launch");
      errors = launch.errors;
    } catch {
      errors = ["Не удалось получить параметры запуска. Перезапустите NimMD."];
    }
    if (request !== generation) return;

    const list = document.createElement("ul");
    list.setAttribute("role", "alert");
    for (const error of errors) {
      const item = document.createElement("li");
      item.textContent = error;
      list.append(item);
    }
    notice.replaceChildren(...(errors.length ? [list] : []));
  }
  root.addEventListener("nimmd-launch-changed", () => { void refresh(); });
  return refresh();
}

export async function showView(root: HTMLElement): Promise<void> {
  const notice = document.createElement("div");
  notice.id = "launch-notice";
  const status = document.createElement("p");
  status.className = "nimmd";
  status.setAttribute("role", "status");
  status.textContent = "Загрузка документа…";
  root.replaceChildren(notice, status);
  const launchReady = watchWindowErrors(root, notice);

  let main;
  try {
    main = documentFragment(await invoke<RenderedView>("get_view"));
  } catch (error) {
    status.setAttribute("role", "alert");
    status.textContent = typeof error === "string" ? error : error instanceof Error ? error.message : viewError;
    await launchReady;
    return;
  }

  status.replaceWith(main);
  await Promise.all([launchReady, renderBlocks(main)]);
}

const root = document.getElementById("app");
if (root) void showView(root);
