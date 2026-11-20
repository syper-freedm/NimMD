import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { EditorView } from "@codemirror/view";
import { documentFragment } from "./document-fragment";
import type { EditorFile, EditorDocument, ReadOnlyReason } from "./editor-file";

export interface Launch {
  mode: "edit" | "view";
  path: string | null;
  errors: string[];
}

function readOnlyFrame(
  document: Document,
  reason: ReadOnlyReason,
  source: string,
): HTMLElement {
  const main = document.createElement("main");
  main.className = "nimmd editor-document";
  const banner = document.createElement("p");
  banner.className = "banner";
  banner.setAttribute("role", "alert");
  banner.textContent =
    reason === "tooLarge"
      ? "Файл слишком большой для редактирования. Показаны первые 10 МиБ."
      : "Кодировка не UTF-8. Редактирование недоступно.";
  const text = document.createElement("pre");
  text.tabIndex = 0;
  text.textContent = source;
  main.append(banner, text);
  return main;
}

async function openEditor(content: HTMLElement): Promise<void> {
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  status.textContent = "Загрузка документа…";
  content.replaceChildren(status);
  let file: EditorFile | undefined;
  let externalPending = false;
  let closingBeforeMount = false;
  const finishClose = async (confirmed: boolean): Promise<void> => {
    await invoke("finish_editor_close", { confirmed });
  };
  const isClosing = () => closingBeforeMount;
  const hasPendingChange = () => externalPending;
  try {
    const webview = getCurrentWebview();
    await Promise.all([
      webview.listen("nimmd-editor-external", () => {
        if (file) void file.refresh();
        else externalPending = true;
      }),
      webview.listen("nimmd-editor-close", () => {
        if (file) {
          void file.close(finishClose);
        } else if (!closingBeforeMount) {
          closingBeforeMount = true;
          void finishClose(true).catch(() => {
            closingBeforeMount = false;
          });
        }
      }),
    ]);
    if (isClosing()) return;

    const payload = await invoke<EditorDocument>("get_editor");
    if (isClosing()) return;
    const firstFrame =
      payload.source === null
        ? readOnlyFrame(
            content.ownerDocument,
            payload.readOnlyReason,
            payload.readOnlyText,
          )
        : documentFragment(payload);
    status.replaceWith(firstFrame);

    if (payload.source !== null) {
      // Между двумя callback WebView успевает показать HTML ядра.
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            resolve();
          });
        });
      });
    }
    if (isClosing()) return;
    const [{ mountEditor, setEditorLocked }, { EditorFile }] =
      await Promise.all([import("./editor"), import("./editor-file")]);
    if (isClosing()) return;
    const notice = document.createElement("div");
    const editorRoot = document.createElement("div");
    editorRoot.className = "nimmd editor-document";
    let view: EditorView | undefined;
    let titleUpdate = Promise.resolve();
    const show = (next: EditorDocument, frame?: HTMLElement) => {
      const focused = view?.hasFocus ?? true;
      view?.destroy();
      view = undefined;
      if (next.source === null) {
        content.replaceChildren(
          notice,
          frame ??
            readOnlyFrame(
              content.ownerDocument,
              next.readOnlyReason,
              next.readOnlyText,
            ),
        );
        return;
      }
      content.replaceChildren(editorRoot);
      editorRoot.replaceChildren(notice);
      view = mountEditor(editorRoot, next.source, {
        windowLabel: next.windowLabel,
        onChange: () => {
          file?.changed();
        },
        onSave: () => {
          void file?.save();
        },
      });
      if (focused) view.focus();
    };
    show(payload, firstFrame);
    file = new EditorFile(notice, payload, {
      read: () => view?.state.doc ?? null,
      replace: show,
      dirty: (dirty) => {
        titleUpdate = titleUpdate
          .then(async () => {
            await invoke("set_document_edited", { edited: dirty });
          })
          .catch(() => {
            // Состояние файла не зависит от того, успело ли окно обновить заголовок.
          });
      },
      lock: (locked) => {
        if (view) setEditorLocked(view, locked);
      },
    });
    if (hasPendingChange()) await file.refresh();
  } catch (error) {
    status.setAttribute("role", "alert");
    status.textContent =
      typeof error === "string"
        ? error
        : "Не удалось открыть редактор. Закройте окно и откройте файл снова.";
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
      heading.textContent =
        launch.mode === "view" ? "Просмотр" : "Редактирование";
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
      const error =
        "Не удалось получить параметры запуска. Перезапустите NimMD.";
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

  root.addEventListener("nimmd-launch-changed", () => {
    void refresh();
  });
  await refresh();
}
