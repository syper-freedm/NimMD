import { invoke } from "@tauri-apps/api/core";
import {
  documentFragment,
  showBlockMessage,
  type RenderedView,
} from "./document-fragment";
import type { Launch } from "./launch";

import {
  diagramTooLong,
  documentLimit,
  maxBlocks,
  maxDiagramLength,
  maxMathLength,
  maxTotalLength,
  renderDiagrams,
  renderMath,
  type Block,
} from "./render-blocks";

const viewError =
  "Не удалось показать документ. Закройте окно и откройте файл снова.";

async function renderBlocks(main: HTMLElement): Promise<void> {
  const math: Block[] = [];
  const diagrams: Block[] = [];
  let total = 0;
  for (const node of main.querySelectorAll<HTMLElement>(
    "[data-math-style], pre.mermaid",
  )) {
    const source = node.textContent;
    const formula = node.hasAttribute("data-math-style");
    if (source.length > (formula ? maxMathLength : maxDiagramLength)) {
      showBlockMessage(
        node,
        formula ? "Формула слишком длинная." : diagramTooLong,
      );
      continue;
    }
    if (
      math.length + diagrams.length >= maxBlocks ||
      total + source.length > maxTotalLength
    ) {
      showBlockMessage(node, documentLimit);
      continue;
    }
    (formula ? math : diagrams).push({ node, source });
    total += source.length;
  }
  await Promise.all([renderMath(math), renderDiagrams(diagrams)]);
}

function watchWindowErrors(
  root: HTMLElement,
  notice: HTMLElement,
): Promise<void> {
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
  root.addEventListener("nimmd-launch-changed", () => {
    void refresh();
  });
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
    status.textContent =
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : viewError;
    await launchReady;
    return;
  }

  status.replaceWith(main);
  await Promise.all([launchReady, renderBlocks(main)]);
}

const root = document.getElementById("app");
if (root) void showView(root);
