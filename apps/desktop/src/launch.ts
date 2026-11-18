import { invoke } from "@tauri-apps/api/core";

export interface Launch {
  mode: "edit" | "view";
  path: string | null;
}

export async function showLaunch(root: HTMLElement): Promise<void> {
  try {
    const launch = await invoke<Launch>("get_launch");
    const heading = document.createElement("h1");
    heading.textContent = launch.mode === "view" ? "Просмотр" : "Редактирование";
    const path = document.createElement("p");
    path.className = "document-path";
    path.textContent = launch.path ?? "Файл не выбран";
    root.dataset.mode = launch.mode;
    root.replaceChildren(heading, path);
  } catch {
    root.setAttribute("role", "alert");
    root.textContent = "Не удалось получить параметры запуска. Перезапустите NimMD.";
  }
}
