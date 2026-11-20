interface MermaidWindow extends Window {
  mermaid?: {
    initialize(options: {
      securityLevel: "strict"; startOnLoad: false; layout: "dagre"; suppressErrorRendering: true;
    }): void;
    run(options: { nodes: HTMLElement[]; suppressErrors: true }): Promise<void>;
  };
}

const libraries: MermaidWindow = window;
let loading: Promise<NonNullable<MermaidWindow["mermaid"]>> | undefined;

async function loadMermaid() {
  if (document.readyState !== "complete") {
    await new Promise<void>(resolve => { window.addEventListener("load", () => { resolve(); }, { once: true }); });
  }
  await new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = "/vendor/mermaid.min.js";
    script.addEventListener("load", () => { resolve(); }, { once: true });
    script.addEventListener("error", () => { reject(new Error("Mermaid недоступен")); }, { once: true });
    document.head.append(script);
  });
  const mermaid = libraries.mermaid;
  if (!mermaid) throw new Error("Mermaid недоступен");
  mermaid.initialize({ securityLevel: "strict", startOnLoad: false, layout: "dagre", suppressErrorRendering: true });
  return mermaid;
}

export function renderDiagram(node: HTMLElement, source: string): void {
  // toDOM вызывается до вставки виджета. Mermaid измеряет размеры уже в документе.
  requestAnimationFrame(() => {
    if (node.isConnected) void draw(node, source);
  });
}

async function draw(node: HTMLElement, source: string): Promise<void> {
  try {
    const mermaid = await (loading ??= loadMermaid());
    if (!node.isConnected) return;
    await mermaid.run({ nodes: [node], suppressErrors: true });
    if (node.querySelector(":scope > svg")) return;
  } catch {
    // При ошибке SVG возвращаем исходник.
  }
  node.textContent = source;
  node.dataset.error = "Не удалось отрисовать диаграмму";
}
