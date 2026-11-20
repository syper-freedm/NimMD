import { invoke } from "@tauri-apps/api/core";
import { documentFragment, type RenderedView } from "./document-fragment";

export interface Launch {
  mode: "edit" | "view";
  path: string | null;
  errors: string[];
}

interface EditorDocument extends RenderedView {
  source: string | null;
}

async function openEditor(content: HTMLElement): Promise<void> {
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  status.textContent = "Загрузка документа…";
  content.replaceChildren(status);
  try {
    const payload = await invoke<EditorDocument>("get_editor");
    const firstFrame = documentFragment(payload);
    status.replaceWith(firstFrame);
    if (payload.source === null) return;

    // Между двумя callback WebView успевает показать HTML ядра.
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => { requestAnimationFrame(() => { resolve(); }); });
    });
    const { mountEditor } = await import("./editor");
    const editorRoot = document.createElement("div");
    editorRoot.className = "editor-document";
    const editor = mountEditor(editorRoot, payload.source);
    firstFrame.replaceWith(editorRoot);
    editor.focus();
  } catch (error) {
    status.setAttribute("role", "alert");
    status.textContent = typeof error === "string" ? error : "Не удалось открыть редактор. Закройте окно и откройте файл снова.";
    content.append(status);
  }
}

export async function showLaunch(root: HTMLElement): Promise<void> {
  const summary = document.createElement("header");
  summary.className = "launch-summary";
  const content = document.createElement("div");
  let opening = false;
  let generation = 0;
  async function refresh(): Promise<void> {
    const request = ++generation;
    try {
      const launch = await invoke<Launch>("get_launch");
      if (request !== generation) return;

      const heading = document.createElement("h1");
      heading.textContent = launch.mode === "view" ? "Просмотр" : "Редактирование";
      const path = document.createElement("p");
      path.className = "document-path";
      path.textContent = launch.path ?? "Файл не выбран";
      root.dataset.mode = launch.mode;
      root.removeAttribute("role");
      if (!summary.parentNode) root.replaceChildren(summary, content);
      summary.replaceChildren(heading, path);
      if (launch.errors.length > 0) {
        const list = document.createElement("ul");
        list.setAttribute("role", "alert");
        for (const error of launch.errors) {
          const item = document.createElement("li");
          item.textContent = error;
          list.append(item);
        }
        summary.append(list);
      }
      if (launch.mode === "edit" && launch.path !== null && !opening) {
        opening = true;
        await openEditor(content);
      }
    } catch {
      if (request !== generation) return;
      const error = "Не удалось получить параметры запуска. Перезапустите NimMD.";
      if (opening) {
        const warning = document.createElement("p");
        warning.setAttribute("role", "alert");
        warning.textContent = error;
        summary.replaceChildren(warning);
      } else {
        root.setAttribute("role", "alert");
        root.textContent = error;
      }
    }
  }

  root.addEventListener("nimmd-launch-changed", () => { void refresh(); });
  await refresh();
}
