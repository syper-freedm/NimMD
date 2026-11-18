import { invoke } from "@tauri-apps/api/core";

export interface Launch {
  mode: "edit" | "view";
  path: string | null;
  errors: string[];
}

export async function showLaunch(root: HTMLElement): Promise<void> {
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
      root.replaceChildren(heading, path);
      if (launch.errors.length > 0) {
        const list = document.createElement("ul");
        list.setAttribute("role", "alert");
        for (const error of launch.errors) {
          const item = document.createElement("li");
          item.textContent = error;
          list.append(item);
        }
        root.append(list);
      }
    } catch {
      if (request !== generation) return;
      root.setAttribute("role", "alert");
      root.textContent = "Не удалось получить параметры запуска. Перезапустите NimMD.";
    }
  }

  root.addEventListener("nimmd-launch-changed", () => { void refresh(); });
  await refresh();
}
