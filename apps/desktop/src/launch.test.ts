import { afterEach, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { showLaunch } from "./launch";

afterEach(clearMocks);

test.each(["edit", "view"] as const)("показывает режим %s и путь из Rust IPC", async (mode) => {
  const calls: string[] = [];
  mockIPC((command) => {
    calls.push(command);
    return { mode, path: "заметка с пробелами.md" };
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
  mockIPC(() => ({ mode: "view", path }));
  const root = document.createElement("main");
  await showLaunch(root);
  expect(root.querySelector("p")?.textContent).toBe(path);
  expect(root.querySelector("img")).toBeNull();
});

test("пустой запуск сообщает, что файл не выбран", async () => {
  mockIPC(() => ({ mode: "edit", path: null }));
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
