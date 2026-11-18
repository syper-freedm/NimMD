import { afterEach, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { showLaunch } from "./launch";

afterEach(clearMocks);

test.each(["edit", "view"] as const)("показывает режим %s и путь из Rust IPC", async (mode) => {
  const calls: string[] = [];
  mockIPC((command) => {
    calls.push(command);
    return { mode, path: "заметка с пробелами.md", errors: [] };
  });
  const root = document.createElement("main");
  await showLaunch(root);
  expect(calls).toEqual(["get_launch"]);
  expect(root.dataset.mode).toBe(mode);
  expect(root.querySelector("h1")?.textContent).toBe(mode === "view" ? "Просмотр" : "Редактирование");
  expect(root.querySelector("p")?.textContent).toBe("заметка с пробелами.md");
  expect(root.querySelector("textarea, [contenteditable]" )).toBeNull();
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
  mockIPC(() => { throw new Error("IPC недоступен"); });
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.getAttribute("role")).toBe("alert");
  expect(root.textContent).toContain("Не удалось получить параметры запуска");
});

test("показывает ошибку открытия, сохраняя путь и режим окна", async () => {
  const error = 'Не удалось открыть «<img src=x onerror=alert(1)>». Выберите файл.';
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
    expect(Array.from(root.querySelectorAll('[role="alert"] li'), (item) => item.textContent)).toEqual(errors);
  });
  expect(root.querySelector(".document-path")?.textContent).toBe("Файл не выбран");
});
