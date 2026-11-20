import { Text } from "@codemirror/state";
import { invoke } from "@tauri-apps/api/core";
import type { RenderedView } from "./document-fragment";

interface FileFormat {
  bom: boolean;
  crlf: boolean;
}

export type ReadOnlyReason = "tooLarge" | "notUtf8";

export type EditorDocument = RenderedView & {
  revision: string | null;
  watchWarning: string | null;
  format?: FileFormat | null;
} & (
    | { source: string; readOnlyReason: null; readOnlyText: null }
    | { source: null; readOnlyReason: ReadOnlyReason; readOnlyText: string }
  );

type SaveReply =
  | { status: "saved"; revision: string; inPlace: boolean }
  | { status: "conflict" }
  | { status: "failed"; message: string; recovery: string | null };

interface EditorBuffer {
  read(): Text | null;
  replace(document: EditorDocument): void;
  dirty(dirty: boolean): void;
  lock?(locked: boolean): void;
}

type Problem =
  { kind: "conflict" } | { kind: "failed"; message: string; reading?: true };

export function normalizeCrlf(source: string): string {
  return source.replaceAll("\r\n", "\n");
}

export function editorText(source: string): Text {
  return Text.of(normalizeCrlf(source).split("\n"));
}

function sameDraft(left: Text | null, right: Text | null): boolean {
  return left === right || (left !== null && right !== null && left.eq(right));
}

type CloseAnswer = (confirmed: boolean) => void | Promise<void>;

function messageFrom(error: unknown): string {
  return typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : "Неизвестная ошибка";
}

export class EditorFile {
  private saved: Text | null;
  private revision: string | null;
  private format: FileFormat | undefined;
  private timer: number | undefined;
  private queue = Promise.resolve();
  private problem: Problem | undefined;
  private noticeText: string | undefined;
  private closing: CloseAnswer | undefined;
  private closed = false;
  private refreshQueued = false;
  private dirty = false;

  constructor(
    private readonly notice: HTMLElement,
    initial: Pick<EditorDocument, "revision" | "watchWarning" | "format">,
    private readonly buffer: EditorBuffer,
  ) {
    this.saved = buffer.read();
    this.revision = initial.revision;
    this.format = initial.format ?? undefined;
    this.noticeText = initial.watchWarning ?? undefined;
    this.renderNotice();
  }

  changed(): void {
    this.updateDirty();
    this.clearTimer();
    if (this.closing && !this.dirty) {
      void this.enqueue(() => this.finishIfSaved());
    } else if (this.dirty && !this.problem && !this.closed) {
      this.timer = window.setTimeout(() => {
        this.timer = undefined;
        void this.enqueue(async () => {
          if (this.timer === undefined) await this.write();
        });
      }, 500);
    }
  }

  save(): Promise<void> {
    this.clearTimer();
    return this.enqueue(async () => {
      await this.write(true);
      await this.finishIfSaved();
    });
  }

  refresh(): Promise<void> {
    if (this.refreshQueued) return this.queue;
    this.refreshQueued = true;
    return this.enqueue(async () => {
      this.refreshQueued = false;
      const next = await this.readDisk();
      if (next === undefined) return;
      if (next.revision !== null && next.revision === this.revision) {
        if (this.problem?.kind === "failed" && this.problem.reading) {
          this.problem = undefined;
          this.changed();
          this.renderNotice();
        }
        return;
      }
      if (this.dirty) {
        this.setProblem({ kind: "conflict" });
      } else {
        this.replace(next);
      }
    });
  }

  close(answer: CloseAnswer): Promise<void> {
    this.clearTimer();
    return this.enqueue(async () => {
      this.closing ??= answer;
      await this.finishIfSaved();
      this.renderNotice();
    });
  }

  private enqueue(action: () => Promise<void>): Promise<void> {
    this.queue = this.queue
      .then(async () => {
        if (!this.closed) await action();
      })
      .catch((error: unknown) => {
        this.setProblem({ kind: "failed", message: messageFrom(error) });
      });
    return this.queue;
  }

  private clearTimer(): void {
    if (this.timer !== undefined) window.clearTimeout(this.timer);
    this.timer = undefined;
  }

  private updateDirty(): void {
    const dirty = !sameDraft(this.buffer.read(), this.saved);
    if (dirty !== this.dirty) {
      this.dirty = dirty;
      this.buffer.dirty(dirty);
    }
  }

  private setProblem(problem: Problem): void {
    this.clearTimer();
    this.problem = problem;
    this.renderNotice();
  }

  private async write(retry = false): Promise<void> {
    const source = this.buffer.read();
    if (
      source === null ||
      sameDraft(source, this.saved) ||
      this.problem?.kind === "conflict" ||
      (this.problem && !retry)
    )
      return;

    if (this.revision === null) {
      this.setProblem({
        kind: "failed",
        message: "Текущая версия файла неизвестна.",
      });
      return;
    }
    let reply: SaveReply;
    try {
      reply = await invoke<SaveReply>("save_editor", {
        source: source.toString(),
        revision: this.revision,
        ...(this.format ? { format: this.format } : {}),
      });
    } catch (error) {
      this.setProblem({ kind: "failed", message: messageFrom(error) });
      return;
    }
    switch (reply.status) {
      case "saved":
        this.saved = source;
        this.revision = reply.revision;
        this.problem = undefined;
        if (reply.inPlace) this.noticeText = "Файл сохранён на месте.";
        this.updateDirty();
        this.renderNotice();
        break;
      case "conflict":
        this.setProblem({ kind: "conflict" });
        break;
      case "failed":
        if (reply.recovery)
          this.noticeText = `Копия сохранена: ${reply.recovery}`;
        this.setProblem({ kind: "failed", message: reply.message });
        break;
    }
  }

  private async readDisk(): Promise<EditorDocument | undefined> {
    try {
      return await invoke<EditorDocument>("read_editor");
    } catch (error) {
      if (this.problem) {
        this.noticeText = `Не удалось перечитать файл. ${messageFrom(error)}`;
        this.renderNotice();
      } else {
        this.setProblem({
          kind: "failed",
          message: messageFrom(error),
          reading: true,
        });
      }
    }
  }

  private replace(document: EditorDocument): void {
    this.buffer.replace(document);
    this.saved = this.buffer.read();
    this.revision = document.revision;
    this.format = document.format ?? undefined;
    this.problem = undefined;
    this.noticeText = document.watchWarning ?? undefined;
    this.updateDirty();
    this.renderNotice();
  }

  private async resolveConflict(keep: boolean): Promise<void> {
    if (this.problem?.kind !== "conflict") return;
    const before = this.buffer.read();
    const next = await this.readDisk();
    if (!next) return;
    if (keep) {
      if (next.source === null || next.revision === null) {
        this.noticeText =
          "Файл на диске недоступен для записи. Правки можно сохранить в копию.";
        this.renderNotice();
        return;
      }
      this.revision = next.revision;
      this.problem = undefined;
      // После явного выбора сверяем запись с диском, даже если пользователь отменил свои правки.
      this.saved = editorText(next.source);
      this.updateDirty();
      await this.write();
    } else {
      // Перечитать подтверждено для текста на момент нажатия кнопки.
      if (!sameDraft(before, this.buffer.read())) return;
      this.replace(next);
    }
    await this.finishIfSaved();
    this.renderNotice();
  }

  private async recover(): Promise<void> {
    const source = this.buffer.read();
    if (source === null) return;
    try {
      const path = await invoke<string>("recover_editor", {
        source: source.toString(),
      });
      this.noticeText = `Копия сохранена: ${path}`;
    } catch (error) {
      this.noticeText = `Не удалось сохранить копию. ${messageFrom(error)}`;
    }
    this.renderNotice();
  }

  private async finishIfSaved(): Promise<void> {
    if (!this.closing) return;
    this.buffer.lock?.(true);
    try {
      while (this.dirty && !this.problem) await this.write();
      if (!this.dirty) await this.finish(true);
    } finally {
      if (!this.closed) this.buffer.lock?.(false);
    }
  }

  private async finish(confirmed: boolean): Promise<void> {
    const answer = this.closing;
    if (!answer) return;
    try {
      await answer(confirmed);
      this.closed = confirmed;
      this.closing = undefined;
      this.renderNotice();
    } catch (error) {
      this.noticeText = `Не удалось завершить закрытие окна. ${messageFrom(error)}`;
      this.renderNotice();
    }
  }

  private renderNotice(): void {
    const { ownerDocument } = this.notice;
    this.notice.replaceChildren();
    const line = (role: "alert" | "status", text: string) => {
      const item = ownerDocument.createElement("p");
      item.setAttribute("role", role);
      item.textContent = text;
      this.notice.append(item);
    };
    const button = (text: string, action: () => Promise<void>) => {
      const element = ownerDocument.createElement("button");
      element.type = "button";
      element.textContent = text;
      element.addEventListener("click", () => {
        void this.enqueue(action);
      });
      this.notice.append(element);
    };

    if (this.problem || (this.closing && this.dirty)) {
      this.notice.className = "banner";
    } else {
      this.notice.className = "";
    }
    if (this.problem?.kind === "conflict") {
      line("alert", "Файл изменён на диске.");
      button("Перечитать", () => this.resolveConflict(false));
      button("Оставить моё", () => this.resolveConflict(true));
    } else if (this.problem) {
      const action = this.problem.reading ? "перечитать" : "сохранить";
      line("alert", `Не удалось ${action} файл. ${this.problem.message}`);
    }
    if (this.problem && this.dirty)
      button("Сохранить копию", () => this.recover());
    if (this.noticeText) line("status", this.noticeText);
    if (this.closing && this.dirty) {
      line("alert", "Закрыть окно без сохранения правок?");
      button("Закрыть без сохранения", async () => {
        this.buffer.lock?.(true);
        await this.finish(true);
        if (!this.closed) this.buffer.lock?.(false);
      });
      button("Отмена", () => this.finish(false));
    }
  }
}
