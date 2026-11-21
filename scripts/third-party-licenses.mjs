import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { allowedLicenses, licenseErrors, npmLicenses } from "./check-npm-licenses.mjs";

const escapeHtml = (text) =>
  text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");

export function npmNotices(byLicense) {
  const notices = new Map();
  for (const [license, packages] of Object.entries(byLicense)) {
    for (const pkg of packages) {
      // Esbuild и его платформенный бинарник используются только при сборке Mermaid.
      if (pkg.name === "esbuild" || pkg.name.startsWith("@esbuild/")) continue;
      for (const path of pkg.paths) {
        const { version } = JSON.parse(readFileSync(join(path, "package.json"), "utf8"));
        const name = `${pkg.name} ${version}`;
        if (notices.has(name)) continue;
        const files = readdirSync(path, { withFileTypes: true })
          .filter((file) => file.isFile() && /^(licen[cs]es?|copying|notice)([._-]|$)/i.test(file.name))
          .map((file) => file.name)
          .sort();
        if (!files.some((file) => !/^notice/i.test(file))) {
          throw new Error(`${name}: файл лицензии не найден`);
        }
        const text = files.map((file) => readFileSync(join(path, file), "utf8")).join("\n\n");
        notices.set(name, { name, license, text });
      }
    }
  }
  return [...notices.values()].sort((a, b) => a.name.localeCompare(b.name, "en"));
}

export function renderLicenses(rust, npm, assets = []) {
  const rustNotices = rust.flatMap(({ id, name, text, used_by: usedBy }) => {
    const packages = usedBy.filter(({ crate }) => crate.source != null);
    if (!packages.length) return [];
    const items = packages.map(({ crate }) => {
      const label = escapeHtml(`${crate.name} ${crate.version}`);
      if (id !== "MPL-2.0") return `<li>${label}</li>`;
      const archive = `https://crates.io/api/v1/crates/${encodeURIComponent(crate.name)}/${encodeURIComponent(crate.version)}/download`;
      return `<li>${label} (<a href="${archive}">исходный код</a>)</li>`;
    });
    return `<details><summary>${escapeHtml(name)}</summary><ul>${items.join("")}</ul><pre>${escapeHtml(text)}</pre></details>`;
  });
  const notices = (entries) =>
    entries.map(({ name, license, text }) =>
      `<details><summary>${escapeHtml(name)} — ${escapeHtml(license)}</summary><pre>${escapeHtml(text)}</pre></details>`,
    ).join("\n");
  return `<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>Лицензии сторонних компонентов NimMD</title>
<style>
:root { color-scheme: light dark; }
body { font: 1rem/1.5 system-ui, sans-serif; max-width: 64rem; margin: 2rem auto; padding: 0 1rem; }
summary { cursor: pointer; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; }
</style>
</head>
<body>
<h1>Лицензии сторонних компонентов NimMD</h1>
<h2>Rust</h2>
<p><a href="RUST_STDLIB_COPYRIGHT.html">Лицензии стандартной библиотеки Rust</a></p>
${rustNotices.join("\n")}
<h2>npm</h2>
${notices(npm)}
<h2>Встроенные ресурсы</h2>
${notices(assets)}
</body>
</html>
`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = join(import.meta.dirname, "..");
  const read = (file) => readFileSync(join(root, file), "utf8");
  const allowed = allowedLicenses(read("deny.toml"));
  const dependencies = {};
  for (const [dir, prod] of [["apps/desktop", true], ["packages/vendor", false]]) {
    const inventory = npmLicenses(join(root, dir), prod);
    if (!Object.keys(inventory).length) throw new Error(`${dir}: зависимости не установлены`);
    const errors = licenseErrors(dir, inventory, allowed);
    if (errors.length) throw new Error(errors.join("\n"));
    for (const [license, packages] of Object.entries(inventory)) {
      (dependencies[license] ??= []).push(...packages);
    }
  }
  const { licenses } = JSON.parse(execFileSync("cargo", [
    "about", "generate", "--config", "about.toml", "--format", "json",
    "--workspace", "--features", "nimmd-desktop/custom-protocol", "--locked", "--fail",
  ], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  }));
  const sysroot = execFileSync(process.env.RUSTC ?? "rustc", ["--print", "sysroot"], {
    cwd: root,
    encoding: "utf8",
  }).trim();
  const stdlib = join(sysroot, "share/doc/rust/COPYRIGHT-library.html");
  if (!existsSync(stdlib)) {
    throw new Error(`Не найдено лицензионное уведомление стандартной библиотеки Rust: ${stdlib}`);
  }
  const vite = JSON.parse(read("apps/desktop/node_modules/vite/package.json"));
  const assets = [
    {
      name: "Шрифты KaTeX",
      license: "OFL-1.1",
      text: read("packages/vendor/katex-fonts-OFL.txt"),
    },
    {
      name: "Mermaid",
      license: "Уведомления из JavaScript-бандла",
      text: read("packages/vendor/dist/mermaid.min.js.LEGAL.txt"),
    },
    {
      name: "Грамматики Sublime Text, включая TOML",
      license: "Sublime HQ Packages",
      text: read("crates/nimmd-core/assets/syntaxes/LICENSE"),
    },
    {
      name: "Грамматики Rust и Cargo из Sublime Text",
      license: "MIT",
      text: read("crates/nimmd-core/assets/syntaxes/syntect-rust-LICENSE.txt"),
    },
    {
      name: `Vite ${vite.version} (полифил modulepreload)`,
      license: vite.license,
      text: read("apps/desktop/node_modules/vite/LICENSE.md"),
    },
  ];
  const output = join(root, "target");
  mkdirSync(output, { recursive: true });
  copyFileSync(stdlib, join(output, "RUST_STDLIB_COPYRIGHT.html"));
  writeFileSync(join(output, "THIRD_PARTY_LICENSES.html"), renderLicenses(licenses, npmNotices(dependencies), assets));
}
