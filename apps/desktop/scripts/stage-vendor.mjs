import { cpSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";

const source = fileURLToPath(
  new URL("../../../packages/vendor/dist/", import.meta.url),
);
const target = fileURLToPath(
  new URL("../.vendor-public/vendor/", import.meta.url),
);

// Публикуем ровно этот набор ресурсов, включая лицензии Mermaid и шрифты KaTeX.
rmSync(target, { force: true, recursive: true });
for (const asset of [
  "katex.min.css",
  "katex.min.js",
  "fonts",
  "mermaid.min.js",
  "mermaid.min.js.LEGAL.txt",
]) {
  cpSync(`${source}${asset}`, `${target}${asset}`, { recursive: true });
}
