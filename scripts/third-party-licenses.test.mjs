import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { npmNotices, renderLicenses } from "./third-party-licenses.mjs";

test("версионная ссылка на исходники MPL и безопасное отображение текста лицензии", () => {
  const html = renderLicenses([
    {
      name: "Mozilla Public License 2.0",
      id: "MPL-2.0",
      text: "Copyright <script>&",
      used_by: [{ crate: { name: "cssparser", version: "0.37.0", source: "registry+https://github.com/rust-lang/crates.io-index" } }],
    },
  ], []);

  assert.match(html, /cssparser 0\.37\.0/);
  assert.match(html, /https:\/\/crates\.io\/api\/v1\/crates\/cssparser\/0\.37\.0\/download/);
  assert.match(html, /Copyright &lt;script&gt;&amp;/);
  assert.doesNotMatch(html, /<script>/);
  assert.match(html, /<a href="RUST_STDLIB_COPYRIGHT\.html">/);
});

test("в уведомления npm входят файлы каждой установленной версии", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "nimmd-licenses-"));
  t.after(() => rmSync(directory, { force: true, recursive: true }));
  const paths = ["1.0.0", "2.0.0"].map((version) => {
    const path = join(directory, version);
    mkdirSync(path);
    writeFileSync(join(path, "package.json"), JSON.stringify({ name: "example", version }));
    writeFileSync(join(path, "LICENSE"), `License ${version}`);
    return path;
  });
  writeFileSync(join(paths[1], "NOTICE.txt"), "Example copyright");

  const notices = npmNotices({ MIT: [{ name: "example", versions: ["1.0.0", "2.0.0"], paths }] });
  const html = renderLicenses([], notices);
  assert.match(html, /example 1\.0\.0/);
  assert.match(html, /example 2\.0\.0/);
  assert.match(html, /License 1\.0\.0/);
  assert.match(html, /License 2\.0\.0/);
  assert.match(html, /Example copyright/);

  rmSync(join(paths[0], "LICENSE"));
  assert.throws(() => npmNotices({ MIT: [{ name: "example", versions: ["1.0.0", "2.0.0"], paths }] }), /example 1\.0\.0/);
});
