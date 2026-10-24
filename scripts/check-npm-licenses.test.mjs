import { test } from "node:test";
import assert from "node:assert/strict";
import { allowedLicenses, isAllowed } from "./check-npm-licenses.mjs";

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
