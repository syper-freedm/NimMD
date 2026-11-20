import { afterEach, expect, test, vi } from "vitest";
import { emit } from "@tauri-apps/api/event";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { EditorView } from "@codemirror/view";
import type { EditorDocument } from "./editor-file";
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
  mockWindows("nimmd-window-1");
  mockIPC(
    (command) =>
      command === "get_launch"
        ? { mode: "edit", path: "note.md", errors }
        : {
            body: '<main class="nimmd"><h1>Первый кадр</h1></main>',
            windowLabel: "nimmd-window-1",
            source: "# Исходник",
            revision: "a",
            watchWarning: null,
          },
    { shouldMockEvents: true },
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

const readOnlyDocument = {
  body: '<main class="nimmd"><h1>Отрисованный документ</h1></main>',
  windowLabel: "nimmd-window-1",
  source: null,
  revision: null,
  watchWarning: null,
  format: null,
  readOnlyText: "# Исходник\n\n**жирный** <img src=x> �",
};

test.each([
  ["tooLarge", "Файл слишком большой для редактирования"],
  ["notUtf8", "Кодировка не UTF-8"],
])(
  "показывает причину %s и отключает редактирование и сохранение",
  async (readOnlyReason, banner) => {
    const calls: string[] = [];
    mockWindows("nimmd-window-1");
    mockIPC(
      (command) => {
        calls.push(command);
        if (command === "get_launch")
          return { mode: "edit", path: "note.md", errors: [] };
        if (command === "get_editor")
          return { ...readOnlyDocument, readOnlyReason };
        if (command === "finish_editor_close") return;
        throw new Error(command);
      },
      { shouldMockEvents: true },
    );

    const root = document.createElement("div");
    await showLaunch(root);
    expect(root.querySelector('[role="alert"]')?.textContent).toContain(banner);
    expect(root.querySelector("pre")?.textContent).toBe(
      "# Исходник\n\n**жирный** <img src=x> �",
    );
    expect(
      root.querySelector("main h1, img, .cm-editor, [contenteditable=true]"),
    ).toBeNull();

    root.querySelector("pre")?.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "s",
        metaKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
    await emit("nimmd-editor-close");
    await vi.waitFor(() => {
      expect(calls).toEqual([
        "get_launch",
        "get_editor",
        "finish_editor_close",
      ]);
    });
  },
);

test("переключает чтение и редактирование по внешним изменениям", async () => {
  const editable = {
    ...readOnlyDocument,
    source: "# Можно править",
    revision: "b",
    format: { bom: false, crlf: false },
    readOnlyReason: null,
    readOnlyText: null,
  };
  const initial: EditorDocument = {
    ...readOnlyDocument,
    readOnlyReason: "tooLarge",
  };
  let disk: EditorDocument = editable;
  mockWindows("nimmd-window-1");
  mockIPC(
    (command) => {
      if (command === "get_launch")
        return { mode: "edit", path: "note.md", errors: [] };
      if (command === "get_editor") return initial;
      if (command === "read_editor") return disk;
      throw new Error(command);
    },
    { shouldMockEvents: true },
  );

  const root = document.createElement("div");
  await showLaunch(root);
  expect(root.querySelector("pre")?.textContent).toBe(
    readOnlyDocument.readOnlyText,
  );
  await emit("nimmd-editor-external");
  await vi.waitFor(() => {
    expect(root.querySelector(".cm-content")?.textContent).toBe(
      "# Можно править",
    );
  });

  disk = {
    ...readOnlyDocument,
    revision: "c",
    readOnlyReason: "notUtf8",
    readOnlyText: "новый �",
  };
  await emit("nimmd-editor-external");
  await vi.waitFor(() => {
    expect(root.querySelector("pre")?.textContent).toBe("новый �");
  });
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

test("дожидается сохранения правок перед ответом на запрос закрытия", async () => {
  mockWindows("nimmd-window-1");
  const frames: FrameRequestCallback[] = [];
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) =>
    frames.push(callback),
  );
  let finishWrite!: (reply: { status: "saved"; revision: string }) => void;
  const write = new Promise((resolve) => {
    finishWrite = resolve;
  });
  const calls: [string, unknown][] = [];
  mockIPC(
    (command, args) => {
      calls.push([command, args]);
      if (command === "get_launch")
        return { mode: "edit", path: "note.md", errors: [] };
      if (command === "get_editor") {
        return {
          body: '<main class="nimmd"><p>Первый кадр</p></main>',
          windowLabel: "nimmd-window-1",
          source: "исходный",
          revision: "a",
          watchWarning: null,
        };
      }
      if (command === "save_editor") return write;
      if (command === "set_document_edited") return null;
      if (command === "finish_editor_close") return null;
      throw new Error(command);
    },
    { shouldMockEvents: true },
  );
  const root = document.createElement("div");
  const ready = showLaunch(root);
  await vi.waitFor(() => {
    expect(frames).toHaveLength(1);
  });
  frames.shift()?.(0);
  frames.shift()?.(16);
  await ready;
  const buffer = root.querySelector(".cm-content");
  expect(buffer).not.toBeNull();
  const paste = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(paste, "clipboardData", {
    value: { getData: () => "правка " },
  });
  buffer?.dispatchEvent(paste);
  expect(paste.defaultPrevented).toBe(true);
  await emit("nimmd-editor-close", null);
  await vi.waitFor(() => {
    expect(calls).toContainEqual([
      "save_editor",
      { source: "правка исходный", revision: "a" },
    ]);
  });
  expect(calls.some(([command]) => command === "finish_editor_close")).toBe(
    false,
  );
  expect(buffer?.getAttribute("contenteditable")).toBe("false");
  finishWrite({ status: "saved", revision: "b" });
  await vi.waitFor(() => {
    expect(calls).toContainEqual(["finish_editor_close", { confirmed: true }]);
  });
});

test("обновляет индикатор окна в порядке быстрых правок", async () => {
  const frames: FrameRequestCallback[] = [];
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) =>
    frames.push(callback),
  );
  const edits: boolean[] = [];
  let releaseFirstUpdate!: () => void;
  const firstUpdate = new Promise<void>((resolve) => {
    releaseFirstUpdate = resolve;
  });
  mockWindows("nimmd-window-1");
  mockIPC(
    (command, args) => {
      if (command === "get_launch")
        return { mode: "edit", path: "note.md", errors: [] };
      if (command === "get_editor") {
        return {
          ...readOnlyDocument,
          source: "исходный",
          revision: "a",
          readOnlyReason: null,
          readOnlyText: null,
          format: { bom: false, crlf: false },
        };
      }
      if (command === "set_document_edited" && args && "edited" in args) {
        edits.push(Boolean(args.edited));
        if (edits.length === 1) return firstUpdate;
        return;
      }
      throw new Error(command);
    },
    { shouldMockEvents: true },
  );
  const root = document.createElement("div");
  const ready = showLaunch(root);
  await vi.waitFor(() => {
    expect(frames).toHaveLength(1);
  });
  frames.shift()?.(0);
  frames.shift()?.(16);
  await ready;
  expect(edits).toEqual([]);
  const element = root.querySelector<HTMLElement>(".cm-editor");
  const editor = element && EditorView.findFromDOM(element);
  if (!editor) throw new Error("Редактор не открылся");

  editor.dispatch({ changes: { from: 0, insert: "+" } });
  await vi.waitFor(() => {
    expect(edits).toEqual([true]);
  });
  editor.dispatch({ changes: { from: 0, to: 1 } });
  await Promise.resolve();
  expect(edits).toEqual([true]);
  releaseFirstUpdate();
  await vi.waitFor(() => {
    expect(edits).toEqual([true, false]);
  });
});
