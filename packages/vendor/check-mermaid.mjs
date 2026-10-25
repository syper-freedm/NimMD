// Проверка собранных ресурсов: в mermaid.min.js нет ELK, состав dist/ ровно ожидаемый.
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "dist");
const bundle = join(dist, "mermaid.min.js");
const upstream = join(here, "node_modules/mermaid/dist/mermaid.min.js");
const expected = ["fonts", "katex.min.css", "katex.min.js", "mermaid.min.js", "mermaid.min.js.LEGAL.txt"];

function fail(message) {
  console.error(message);
  process.exit(1);
}

if (readFileSync(bundle, "utf8").includes("org.eclipse.elk")) fail(`в ${bundle} найдена строка org.eclipse.elk`);
if (statSync(bundle).size >= statSync(upstream).size) fail(`${bundle} не меньше штатного ${upstream}: ELK, похоже, в бандле`);
const pnpmStore = join(here, "node_modules/.pnpm");
const elkPkgs = existsSync(pnpmStore) ? readdirSync(pnpmStore).filter((n) => n.startsWith("elkjs@")) : [];
if (elkPkgs.length) fail(`elkjs установлен: node_modules/.pnpm/${elkPkgs[0]}`);
const actual = readdirSync(dist).sort();
if (actual.join("\n") !== expected.join("\n")) fail(`состав dist/: ${actual.join(", ")}; ожидается: ${expected.join(", ")}`);
if (readdirSync(join(dist, "fonts")).length === 0) fail("dist/fonts/ пуст");

console.log("ok: elkjs не найден");
