import { afterEach, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { showLaunch } from "./launch";

afterEach(() => {
  clearMocks();
  vi.unstubAllGlobals();
});

test("показывает первый кадр из Rust перед монтированием редактора", async () => {
  const frames: FrameRequestCallback[] = [];
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) =>
    frames.push(callback),
  );
  let errors: string[] = [];
  mockIPC((command) =>
    command === "get_launch"
      ? { mode: "edit", path: "note.md", errors }
      : {
          body: '<main class="nimmd"><h1>Первый кадр</h1></main>',
          windowLabel: "nimmd-window-1",
          source: "# Исходник",
        },
  );
  const root = document.createElement("div");
  const ready = showLaunch(root);
  await vi.waitFor(() => {
    expect(frames).toHaveLength(1);
  });
  expect(root.querySelector("main.nimmd h1")?.textContent).toBe("Первый кадр");
  expect(root.querySelector(".cm-editor")).toBeNull();
  frames.shift()?.(0);
  expect(root.querySelector(".cm-editor")).toBeNull();
  frames.shift()?.(16);
  await ready;
  expect(root.querySelector(".cm-content")?.textContent).toBe("# Исходник");
  expect(root.querySelector("main.nimmd")).toBeNull();

  const editor = root.querySelector(".cm-content");
  errors = ["Ошибка другого файла"];
  root.dispatchEvent(new Event("nimmd-launch-changed"));
  await vi.waitFor(() => {
    expect(root.querySelector('[role="alert"]')?.textContent).toBe(errors[0]);
  });
  expect(root.querySelector(".cm-content")).toBe(editor);
});

test("оставляет первый кадр, когда Rust не передал редактируемый текст", async () => {
  mockIPC((command) =>
    command === "get_launch"
      ? { mode: "edit", path: "note.md", errors: [] }
      : {
          body: '<main class="nimmd"><p>Только чтение</p></main>',
          windowLabel: "nimmd-window-1",
          source: null,
        },
  );
  const root = document.createElement("div");
  await showLaunch(root);
  expect(root.querySelector("main.nimmd")?.textContent).toBe("Только чтение");
  expect(root.querySelector(".cm-editor")).toBeNull();
});

test("показывает режим просмотра и путь из Rust IPC", async () => {
  const mode = "view";
  const calls: string[] = [];
  mockIPC((command) => {
    calls.push(command);
    return { mode, path: "заметка с пробелами.md", errors: [] };
  });
  const root = document.createElement("main");
  await showLaunch(root);
  expect(calls).toEqual(["get_launch"]);
  expect(root.dataset.mode).toBe(mode);
  expect(root.querySelector("h1")?.textContent).toBe("Просмотр");
  expect(root.querySelector("p")?.textContent).toBe("заметка с пробелами.md");
  expect(root.querySelector("textarea, [contenteditable]")).toBeNull();
});

test("имя файла остаётся текстом, даже если содержит HTML", async () => {
  const path = '<img src=x onerror="alert(1)">.md';
  mockIPC(() => ({ mode: "view", path, errors: [] }));
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.querySelector("p")?.textContent).toBe(path);
  expect(root.querySelector("img")).toBeNull();
});

test("пустой запуск сообщает, что файл не выбран", async () => {
  mockIPC(() => ({ mode: "edit", path: null, errors: [] }));
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.textContent).toContain("Файл не выбран");
});

test("ошибка IPC видна пользователю", async () => {
  mockIPC(() => {
    throw new Error("IPC недоступен");
  });
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.getAttribute("role")).toBe("alert");
  expect(root.textContent).toContain("Не удалось получить параметры запуска");
});

test("показывает ошибку открытия, сохраняя путь и режим окна", async () => {
  const error =
    "Не удалось открыть «<img src=x onerror=alert(1)>». Выберите файл.";
  mockIPC(() => ({ mode: "view", path: "note.md", errors: [error] }));
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.querySelector('[role="alert"]')?.textContent).toBe(error);
  expect(root.querySelector(".document-path")?.textContent).toBe("note.md");
  expect(root.dataset.mode).toBe("view");
  expect(root.querySelector("img")).toBeNull();
});

test("загруженное окно показывает новую ошибку открытия", async () => {
  let errors: string[] = [];
  mockIPC(() => ({ mode: "edit", path: null, errors }));
  const root = document.createElement("main");
  await showLaunch(root);
  errors = ["Выберите файл.", "Проверьте, что файл существует."];
  root.dispatchEvent(new Event("nimmd-launch-changed"));
  await vi.waitFor(() => {
    expect(
      Array.from(
        root.querySelectorAll('[role="alert"] li'),
        (item) => item.textContent,
      ),
    ).toEqual(errors);
  });
  expect(root.querySelector(".document-path")?.textContent).toBe(
    "Файл не выбран",
  );
});
