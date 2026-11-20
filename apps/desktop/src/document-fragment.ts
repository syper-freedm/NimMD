import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import "../../../packages/theme/nimmd.css";
import "../../../packages/theme/highlight.css";
import "../../../packages/theme/data.css";

export function showBlockMessage(node: HTMLElement, message: string): void {
  const notice = document.createElement("span");
  notice.setAttribute("role", "status");
  notice.textContent = ` ${message}`;
  node.after(notice);
}

function prepareDocument(main: HTMLElement, windowLabel: string): void {
  for (const image of main.querySelectorAll("img")) {
    const src = image.getAttribute("src") ?? "";
    const showPlaceholder = () => {
      showBlockMessage(
        image,
        image.alt
          ? `Картинка недоступна: ${image.alt}`
          : "Картинка недоступна.",
      );
      image.remove();
    };
    if (!src) {
      showPlaceholder();
      continue;
    }
    image.addEventListener("error", showPlaceholder, { once: true });
    if (!/^data:/i.test(src)) {
      const path = src.split(/[?#]/, 1)[0] ?? "";
      const endpoint = convertFileSrc(windowLabel, "nimmd-asset");
      image.src = `${endpoint}?${new URLSearchParams({ path })}`;
    }
  }

  const openLink = (event: MouseEvent) => {
    if (!(event.target instanceof Element) || event.defaultPrevented) return;
    const link = event.target.closest<HTMLAnchorElement>("a[href]");
    const url = link?.getAttribute("href")?.trim();
    if (!link || url?.startsWith("#")) return;
    event.preventDefault();
    if (url && /^(https?:\/\/|mailto:)/i.test(url)) {
      void invoke("open_external", { url }).catch(() => {
        showBlockMessage(link, "Не удалось открыть ссылку.");
      });
    }
  };
  main.addEventListener("click", openLink);
  main.addEventListener("auxclick", (event) => {
    if (event.button === 1) openLink(event);
  });
}

export interface RenderedView {
  body: string;
  windowLabel: string;
}

export function documentFragment({
  body,
  windowLabel,
}: RenderedView): HTMLElement {
  // Ресурсы фрагмента остаются неактивными до обработки ссылок на картинки.
  const parsed = new DOMParser().parseFromString(
    `<template>${body}</template>`,
    "text/html",
  );
  const content = parsed
    .querySelector("template")
    ?.content.querySelector<HTMLElement>("main.nimmd");
  if (!content)
    throw new Error(
      "Не удалось показать документ. Закройте окно и откройте файл снова.",
    );
  const main = document.importNode(content, true);
  prepareDocument(main, windowLabel);
  return main;
}
