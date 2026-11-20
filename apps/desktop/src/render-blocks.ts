import { showBlockMessage } from "./document-fragment";

export type Block = { node: HTMLElement; source: string };

interface ViewLibraries extends Window {
  katex?: {
    render: (
      source: string,
      node: HTMLElement,
      options: {
        displayMode: boolean;
        trust: false;
        throwOnError: false;
        errorColor: string;
      },
    ) => void;
  };
  mermaid?: {
    initialize: (options: {
      securityLevel: "strict";
      startOnLoad: false;
      layout: "dagre";
      theme: "default" | "dark";
      suppressErrorRendering: true;
      deterministicIds: true;
    }) => void;
    run: (options: {
      nodes: HTMLElement[];
      suppressErrors: true;
    }) => Promise<void>;
  };
}

const libraries: ViewLibraries = window;
export const maxMathLength = 4096;
export const maxDiagramLength = 8192;
export const maxBlocks = 32;
export const maxTotalLength = 32_768;
export const documentLimit = "Превышен лимит документа для формул и диаграмм.";
export const diagramTooLong = "Диаграмма слишком длинная.";
const mathErrorColor = "var(--nimmd-katex-error)";

const loadedAssets = new Map<
  string,
  { asset: Element; ready: Promise<void> }
>();

function loadAsset(asset: HTMLLinkElement | HTMLScriptElement): Promise<void> {
  const url = asset instanceof HTMLLinkElement ? asset.href : asset.src;
  const loaded = loadedAssets.get(url);
  if (loaded?.asset.isConnected) return loaded.ready;
  const ready = appendAsset(asset);
  loadedAssets.set(url, { asset, ready });
  return ready;
}

function appendAsset(
  asset: HTMLLinkElement | HTMLScriptElement,
): Promise<void> {
  return new Promise((resolve, reject) => {
    asset.addEventListener(
      "load",
      () => {
        resolve();
      },
      { once: true },
    );
    asset.addEventListener(
      "error",
      () => {
        asset.remove();
        reject(new Error("Ресурс недоступен"));
      },
      { once: true },
    );
    document.head.append(asset);
  });
}

export async function renderMath(blocks: Block[]): Promise<void> {
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
    for (const { node } of blocks) {
      if (node.isConnected) {
        showBlockMessage(
          node,
          "Не удалось отрисовать формулу. Ресурс недоступен.",
        );
      }
    }
    return;
  }

  for (const { node, source } of blocks) {
    if (!node.isConnected) continue;
    const output = document.createElement("span");
    try {
      libraries.katex.render(source, output, {
        displayMode: node.dataset.mathStyle === "display",
        trust: false,
        throwOnError: false,
        errorColor: mathErrorColor,
      });
      // KaTeX с throwOnError: false помечает неизвестные команды цветом в MathML.
      if (
        output.querySelector(`.katex-error, [mathcolor="${mathErrorColor}"]`)
      ) {
        throw new Error("Ошибка TeX");
      }
      node.replaceChildren(...output.childNodes);
    } catch {
      showBlockMessage(node, "Не удалось отрисовать формулу.");
    }
  }
}

let initializedMermaid: ViewLibraries["mermaid"];
let diagramQueue = Promise.resolve();

export function renderDiagrams(diagrams: Block[]): Promise<void> {
  if (!diagrams.length) return Promise.resolve();
  const next = diagramQueue.then(() =>
    drawDiagrams(diagrams.filter(({ node }) => node.isConnected)),
  );
  diagramQueue = next.catch(() => {});
  return next;
}

async function drawDiagrams(diagrams: Block[]): Promise<void> {
  if (!diagrams.length) return;
  // Самозапуск Mermaid привязан к load. Подключаем скрипт после события.
  if (document.readyState !== "complete") {
    await new Promise<void>((resolve) => {
      window.addEventListener(
        "load",
        () => {
          resolve();
        },
        { once: true },
      );
    });
  }

  const script = document.createElement("script");
  script.src = "/vendor/mermaid.min.js";
  let mermaid;
  try {
    await loadAsset(script);
    mermaid = libraries.mermaid;
    if (!mermaid) throw new Error("Mermaid недоступен");
    if (initializedMermaid !== mermaid)
      mermaid.initialize({
        securityLevel: "strict",
        startOnLoad: false,
        layout: "dagre",
        theme: window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "default",
        suppressErrorRendering: true,
        deterministicIds: true,
      });
    initializedMermaid = mermaid;
  } catch {
    for (const { node } of diagrams) {
      if (node.isConnected) {
        showBlockMessage(
          node,
          "Не удалось отрисовать диаграмму. Ресурс недоступен.",
        );
      }
    }
    return;
  }
  const connected = diagrams.filter(({ node }) => node.isConnected);
  if (!connected.length) return;
  try {
    await mermaid.run({
      nodes: connected.map(({ node }) => node),
      suppressErrors: true,
    });
  } catch {
    // Результат и исходник проверяются для каждого блока отдельно.
  }
  for (const { node, source } of connected) {
    if (!node.isConnected || node.querySelector(":scope > svg")) continue;
    node.textContent = source;
    node.removeAttribute("data-processed");
    showBlockMessage(node, "Не удалось отрисовать диаграмму.");
  }
}
