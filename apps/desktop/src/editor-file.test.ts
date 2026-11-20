import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, expect, test, vi } from "vitest";
import { EditorFile, editorText } from "./editor-file";

afterEach(() => {
  clearMocks();
  vi.useRealTimers();
});

function fixture() {
  const view = document.createElement("div");
  let text = "исходный";
  let revision = "a";
  let disk = text;
  let externalVersion = 0;
  const calls: string[] = [];
  const file = new EditorFile(
    view,
    { revision, watchWarning: null },
    {
      read: () => editorText(text),
      replace: (doc) => {
        text = doc.source ?? "";
      },
      dirty: () => {},
    },
  );
  mockIPC((command, value) => {
    calls.push(command);
    if (command === "read_editor") return { source: disk, revision };
    if (command === "save_editor") {
      const args = value as { source: string; revision: string };
      if (args.revision !== revision) return { status: "conflict" };
      revision = "updated";
      disk = args.source;
      return { status: "saved", revision };
    }
    throw new Error(command);
  });
  return {
    file,
    view,
    calls,
    disk: () => disk,
    edit: (next: string) => {
      text = next;
      file.changed();
    },
    external: (next: string) => {
      disk = next;
      revision = `external-${String(++externalVersion)}`;
    },
  };
}

test("не пишет без правок и автоматически сохраняет после 500 мс", async () => {
  vi.useFakeTimers();
  const { file, calls, disk, edit } = fixture();
  await file.save();
  expect(calls).toEqual([]);
  edit("правка");
  await vi.advanceTimersByTimeAsync(499);
  expect(calls).toEqual([]);
  await vi.advanceTimersByTimeAsync(1);
  expect(calls).toEqual(["save_editor"]);
  expect(disk()).toBe("правка");
});

test("конфликт при закрытии оставляет правку и позволяет осознанно сохранить её", async () => {
  const { file, view, disk, edit, external, calls } = fixture();
  edit("моя правка");
  external("чужая правка");
  const closed = vi.fn();
  await file.close(closed);
  expect(closed).not.toHaveBeenCalled();
  expect(disk()).toBe("чужая правка");
  expect(view.textContent).toContain("Файл изменён на диске");
  expect(view.textContent).toContain("Закрыть без сохранения");
  [...view.querySelectorAll("button")]
    .find((button) => button.textContent === "Оставить моё")
    ?.click();
  await vi.waitFor(() => {
    expect(disk()).toBe("моя правка");
  });
  expect(calls).toEqual(["save_editor", "read_editor", "save_editor"]);
  await file.close(closed);
  expect(closed).toHaveBeenCalledOnce();
});

test("закрытие после записи в полёте сохраняет более свежую правку и блокирует ввод", async () => {
  const view = document.createElement("div");
  let draft = editorText("исходный");
  let locked = false;
  const calls: unknown[] = [];
  let completeFirst!: (reply: { status: "saved"; revision: string }) => void;
  const first = new Promise((resolve) => {
    completeFirst = resolve;
  });
  mockIPC((command, value) => {
    expect(command).toBe("save_editor");
    calls.push(value);
    return calls.length === 1 ? first : { status: "saved", revision: "c" };
  });
  const file = new EditorFile(
    view,
    { revision: "a", watchWarning: null, format: { bom: true, crlf: true } },
    {
      read: () => draft,
      replace: () => {},
      dirty: () => {},
      lock: (value) => {
        locked = value;
      },
    },
  );
  draft = editorText("первая правка");
  file.changed();
  const writing = file.save();
  await vi.waitFor(() => {
    expect(calls).toHaveLength(1);
  });

  draft = editorText("вторая правка\nпродолжение");
  file.changed();
  const closed = vi.fn((confirmed: boolean) => {
    expect(confirmed).toBe(true);
    expect(locked).toBe(true);
  });
  const closing = file.close(closed);
  completeFirst({ status: "saved", revision: "b" });
  await Promise.all([writing, closing]);
  expect(calls).toEqual([
    {
      source: "первая правка",
      revision: "a",
      format: { bom: true, crlf: true },
    },
    {
      source: "вторая правка\nпродолжение",
      revision: "b",
      format: { bom: true, crlf: true },
    },
  ]);
  expect(closed).toHaveBeenCalledOnce();
});

test("перечитывает чистый документ и сохраняет локальный текст при следующем внешнем изменении", async () => {
  const { file, view, calls, edit, external } = fixture();
  external("обновлённый файл");
  await file.refresh();
  await file.save();
  expect(calls).toEqual(["read_editor"]);
  expect(view.textContent).toBe("");

  edit("локальные правки");
  external("ещё одна внешняя версия");
  await file.refresh();
  expect(view.textContent).toContain("Файл изменён на диске");
  expect(view.textContent).toContain("Перечитать");
  expect(view.textContent).toContain("Сохранить копию");
});

test("после ошибки записи сохраняет резервную копию и отменяет закрытие", async () => {
  const { file, view, edit } = fixture();
  mockIPC((command, args) => {
    if (command === "save_editor")
      return { status: "failed", message: "нет места", recovery: null };
    if (command === "recover_editor") {
      expect(args).toEqual({ source: "важная правка" });
      return "/tmp/копия.md";
    }
    throw new Error(command);
  });
  edit("важная правка");
  const answer = vi.fn();
  await file.close(answer);
  expect(view.textContent).toContain("нет места");
  const click = (name: string) => {
    const button = [...view.querySelectorAll("button")].find(
      (item) => item.textContent === name,
    );
    expect(button).toBeDefined();
    button?.click();
  };
  click("Сохранить копию");
  await vi.waitFor(() => {
    expect(view.textContent).toContain("/tmp/копия.md");
  });
  click("Отмена");
  await vi.waitFor(() => {
    expect(answer).toHaveBeenCalledExactlyOnceWith(false);
  });
  expect(view.textContent).not.toContain("Закрыть без сохранения");
});
