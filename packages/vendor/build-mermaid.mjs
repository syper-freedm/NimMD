// Сборка mermaid.min.js без elkjs: импорты elkjs подменяются на elk-stub.js (спек 12).
import { build } from "esbuild";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const elkStub = join(here, "elk-stub.js");

const result = await build({
  absWorkingDir: here,
  entryPoints: [join(here, "mermaid-entry.js")],
  bundle: true,
  minify: true,
  format: "iife",
  globalName: "mermaid",
  footer: { js: "mermaid=mermaid.default;" },
  outfile: join(here, "dist/mermaid.min.js"),
  metafile: true,
  legalComments: "external",
  plugins: [
    {
      name: "elk-stub",
      setup(b) {
        b.onResolve({ filter: /^elkjs(\/|$)/ }, () => ({ path: elkStub }));
      },
    },
  ],
});

const elk = Object.keys(result.metafile.inputs).filter((p) => p.replaceAll("\\", "/").includes("node_modules/elkjs/"));
if (elk.length) {
  console.error(`elkjs попал во входы сборки:\n${elk.join("\n")}`);
  process.exit(1);
}
