import { test } from "node:test";
import assert from "node:assert/strict";
import { allowedLicenses, isAllowed, reconcile } from "./check-npm-licenses.mjs";

const allowed = allowedLicenses(`[licenses]
allow = ["MIT", "Apache-2.0",
         "ISC"]
exceptions = [
  { allow = ["MPL-2.0"], crate = "cssparser" },
]`);

test("белый список берётся из [licenses] allow, а не из исключений", () => {
  assert.deepEqual([...allowed], ["MIT", "Apache-2.0", "ISC"]);
});

test("SPDX-выражения", () => {
  assert.ok(isAllowed("MIT", allowed));
  assert.ok(isAllowed("(MIT OR GPL-3.0)", allowed));
  assert.ok(isAllowed("MIT AND ISC", allowed));
  assert.ok(!isAllowed("EPL-2.0", allowed));
  assert.ok(!isAllowed("MIT AND GPL-3.0", allowed));
  assert.ok(!isAllowed("Unknown", allowed));
  assert.ok(!isAllowed("MIT AND (ISC OR GPL-3.0)", allowed));
});

// Нарушения reconcile — без префикса каталога: его добавляет check (ARCH-001).
const khroma = { "khroma@2.1.0": "MIT" };

test("SC-001: уточнение закрывает Unknown", () => {
  const out = reconcile(
    { Unknown: [{ name: "khroma", versions: ["2.1.0"] }], MIT: [{ name: "a", versions: ["1.0.0"] }] },
    allowed,
    khroma,
  );
  assert.deepEqual(out, { violations: [], unused: [] });
});

test("SC-002: новая версия снова на проверку", () => {
  const out = reconcile({ Unknown: [{ name: "khroma", versions: ["2.1.1"] }] }, allowed, khroma);
  assert.equal(out.violations.length, 1);
  assert.ok(out.violations[0].endsWith("khroma@2.1.1 — Unknown"), out.violations[0]);
  assert.deepEqual(out.unused, ["khroma@2.1.0"]);
});

test("SC-003: уточнение при известной лицензии считается использованным", () => {
  const out = reconcile({ MIT: [{ name: "khroma", versions: ["2.1.0"] }] }, allowed, khroma);
  assert.deepEqual(out, { violations: [], unused: [] });
});

test("SC-004: уточнение не обходит белый список", () => {
  const out = reconcile({ Unknown: [{ name: "x", versions: ["1.0.0", "2.0.0"] }] }, allowed, { "x@1.0.0": "GPL-3.0" });
  assert.equal(out.violations.length, 2);
  assert.ok(out.violations[0].endsWith("x@1.0.0 — Unknown (уточнено: GPL-3.0)"), out.violations[0]);
  assert.ok(out.violations[1].endsWith("x@2.0.0 — Unknown"), out.violations[1]);
  assert.deepEqual(out.unused, []);
});

test("SC-005: без уточнений поведение прежнее", () => {
  const out = reconcile(
    { Unknown: [{ name: "y", versions: ["1.0.0"] }], "Apache-2.0": [{ name: "z", versions: ["3.0.0"] }] },
    allowed,
    {},
  );
  assert.equal(out.violations.length, 1);
  assert.ok(out.violations[0].endsWith("y@1.0.0 — Unknown"), out.violations[0]);
  assert.ok(!out.violations.some((v) => v.includes("z@3.0.0")));
  assert.deepEqual(out.unused, []);
});

test("уточнение одной версии не закрывает соседнюю, пустой вывод pnpm оставляет уточнение неиспользованным", () => {
  const out = reconcile({ Unknown: [{ name: "khroma", versions: ["2.1.0", "2.1.1"] }] }, allowed, khroma);
  assert.deepEqual(out, { violations: ["khroma@2.1.1 — Unknown"], unused: [] });
  assert.deepEqual(reconcile({}, allowed, khroma), { violations: [], unused: ["khroma@2.1.0"] });
});
