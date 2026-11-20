// Сборка mermaid.min.js без elkjs: импорты elkjs подменяются на elk-stub.js (спек 12).
import { build } from "esbuild";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const elkStub = join(here, "elk-stub.js");

// Mermaid 12.2.0 уже разобрал frontmatter и директивы в этой точке.
// Меняем только выбранный layout, сохраняя длину диаграммы и текст подписей.
const layoutSelection =
  "const value = readAppearance(layer, diagramConfigKey, key);";
let replacedLayout = false;
const useDagre = {
  name: "nimmd-mermaid-dagre",
  setup(builder) {
    builder.onLoad(
      {
        filter:
          /[\\/]mermaid[\\/]dist[\\/]chunks[\\/]mermaid\.core[\\/]chunk-DSNLSHJZ\.mjs$/,
      },
      async ({ path }) => {
        const source = await readFile(path, "utf8");
        if (source.split(layoutSelection).length !== 2)
          throw new Error("Изменилось место выбора layout в Mermaid");
        replacedLayout = true;
        return {
          contents: source.replace(
            layoutSelection,
            [
              "const appearance = readAppearance(layer, diagramConfigKey, key);",
              'const value = key === "layout" && typeof appearance === "string" && appearance.startsWith("elk") ? "dagre" : appearance;',
            ].join("\n      "),
          ),
          resolveDir: dirname(path),
        };
      },
    );
  },
};

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
  plugins: [useDagre],
  alias: { "elkjs/lib/elk.bundled.js": elkStub },
});

if (!replacedLayout)
  throw new Error("Не найден модуль выбора layout в Mermaid");

const elk = Object.keys(result.metafile.inputs).filter((p) =>
  p.replaceAll("\\", "/").includes("node_modules/elkjs/"),
);
if (elk.length) {
  console.error(`elkjs попал во входы сборки:\n${elk.join("\n")}`);
  process.exit(1);
}
