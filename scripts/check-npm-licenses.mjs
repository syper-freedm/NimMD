// Сверка лицензий npm-зависимостей с белым списком из deny.toml (спек 12).
// Запуск: node scripts/check-npm-licenses.mjs <каталог> [--prod]
// Каталог без package.json пропускается: фронтенд появится в плане 3.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export function allowedLicenses(denyToml) {
  const list = denyToml.match(/^allow\s*=\s*\[([^\]]*)\]/m);
  if (!list) throw new Error("в deny.toml не найден список [licenses] allow");
  return new Set([...list[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]));
}

// SPDX-выражение из одного идентификатора, «A OR B» или «A AND B».
// Скобки внутри и смесь OR с AND не разбираются — такая лицензия смотрится вручную.
export function isAllowed(expr, allowed) {
  const e = expr.trim().replace(/^\((.*)\)$/, "$1");
  if (allowed.has(e)) return true;
  if (/[()]/.test(e)) return false;
  const or = e.split(/\s+OR\s+/);
  const and = e.split(/\s+AND\s+/);
  if (or.length > 1 && and.length > 1) return false;
  return or.length > 1 ? or.some((l) => allowed.has(l)) : and.every((l) => allowed.has(l));
}

function check(dir, prod, allowed) {
  if (!existsSync(join(dir, "package.json"))) return [];
  const args = ["licenses", "list", "--json", ...(prod ? ["--prod"] : [])];
  const byLicense = JSON.parse(execFileSync("pnpm", args, { cwd: dir, encoding: "utf8", shell: process.platform === "win32" }) || "{}");
  return Object.entries(byLicense)
    .filter(([license]) => !isAllowed(license, allowed))
    .flatMap(([license, pkgs]) => pkgs.map((p) => `${dir}: ${p.name}@${p.versions.join(",")} — ${license}`));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [dir, ...flags] = process.argv.slice(2);
  const root = join(fileURLToPath(import.meta.url), "../..");
  const bad = check(dir, flags.includes("--prod"), allowedLicenses(readFileSync(join(root, "deny.toml"), "utf8")));
  if (bad.length) {
    console.error(`Лицензии вне белого списка deny.toml:\n${bad.join("\n")}`);
    process.exit(1);
  }
}
