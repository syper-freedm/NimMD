import { forceParsing } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { renderDiagram } from "./mermaid";
import { blockWidgets, markdownSyntax } from "./widgets";

const options = new URLSearchParams(location.search);
const proseParagraphs = options.has("sparse") ? 140 : 14;
const section = [
  "Editable paragraph before the previews.\n\n",
  "| Name | Value |\n| --- | --- |\n| Alpha | Beta |\n\n",
  "```mermaid\nflowchart TD\n  A[Alpha] --> B[Beta]\n```\n\n",
  "A paragraph with **strong** text, `code` and a [link](https://example.org).\n\n".repeat(proseParagraphs),
].join("");
const bytes = 1024 * 1024;
const sections = Math.floor(bytes / section.length);
const doc = (section.repeat(sections) + "x".repeat(bytes % section.length)).replace("A[Alpha]", "A[Probe]");
const sampleCount = 60;
const warmupCount = 10;
const scenarios = [
  { name: "Начало, абзац", position: 8 },
  { name: "Конец, абзац", position: (sections - 3) * section.length + 8 },
  { name: "Начало, исходник Mermaid", position: doc.indexOf("A[Probe]") + 2, verifyDiagram: true },
];

const root = document.getElementById("editor");
const run = document.querySelector<HTMLButtonElement>("#run");
const output = document.querySelector<HTMLOutputElement>("#result");
if (!root || !run || !output) throw new Error("Нет элементов стенда");

function frame() {
  return new Promise<void>(resolve => { requestAnimationFrame(() => { resolve(); }); });
}

function statistics(samples: number[]) {
  const ordered = [...samples].sort((a, b) => a - b);
  const percentile = (p: number) => Number(ordered[Math.ceil(p * ordered.length) - 1]?.toFixed(2));
  return { p50: percentile(0.5), p95: percentile(0.95), max: percentile(1) };
}

function mount(parent: HTMLElement, widgets: boolean) {
  return new EditorView({ doc, parent, extensions: [markdownSyntax, widgets ? blockWidgets(renderDiagram) : []] });
}

async function verifyPreviews(view: EditorView, label?: string) {
  const deadline = performance.now() + 8000;
  while (performance.now() < deadline) {
    const tables = view.contentDOM.querySelectorAll("table").length;
    const svgs = [...view.contentDOM.querySelectorAll(".mermaid > svg")];
    const diagrams = svgs.length;
    if (tables > 0 && diagrams > 0 && tables < sections && diagrams < sections &&
        (!label || svgs.some(svg => [...svg.querySelectorAll(".nodeLabel, text")]
          .some(node => node.textContent.trim() === label)))) return { tables, diagrams };
    await frame();
  }
  throw new Error(`В viewport не появились таблица и ожидаемый SVG Mermaid${label ? ` (${label})` : ""}`);
}

async function measure(parent: HTMLElement, widgets: boolean, scenario: typeof scenarios[number]) {
  const view = mount(parent, widgets);
  try {
    if (!forceParsing(view, doc.length, 2000)) throw new Error("Дерево Markdown не готово");
    const { position } = scenario;
    view.dispatch({ selection: { anchor: position }, effects: EditorView.scrollIntoView(position, { y: "start" }) });
    view.focus();
    await frame();
    await frame();
    const visible = widgets ? await verifyPreviews(view) : null;
    if (widgets && scenario.verifyDiagram) {
      view.dispatch(view.state.replaceSelection("x"));
      await verifyPreviews(view, "xProbe");
      view.dispatch({ changes: { from: position, to: position + 1 }, selection: { anchor: position } });
      await verifyPreviews(view, "Probe");
    }
    const dispatch: number[] = [];
    const twoFrames: number[] = [];
    for (let i = 0; i < warmupCount + sampleCount; i++) {
      await frame();
      const start = performance.now();
      if (i % 2 === 0) view.dispatch(view.state.replaceSelection("x"));
      else view.dispatch({ changes: { from: position, to: position + 1 }, selection: { anchor: position } });
      const dispatched = performance.now();
      await frame();
      await frame();
      if (i >= warmupCount) {
        dispatch.push(dispatched - start);
        twoFrames.push(performance.now() - start);
      }
    }
    if (view.state.doc.toString() !== doc) throw new Error("Цикл набора изменил исходник");
    return {
      scenario: scenario.name, widgets, visible,
      dispatchMs: statistics(dispatch), twoFramesMs: statistics(twoFrames),
      dispatchOver16ms: dispatch.filter(ms => ms > 16.7).length,
    };
  } finally {
    view.destroy();
  }
}

async function benchmark(parent: HTMLElement, button: HTMLButtonElement, result: HTMLOutputElement) {
  button.disabled = true;
  result.dataset.state = "running";
  result.textContent = "Идёт замер…";
  try {
    const results = [];
    for (const scenario of scenarios) {
      results.push(await measure(parent, false, scenario));
      results.push(await measure(parent, true, scenario));
    }
    const userAgent = navigator.userAgent;
    result.textContent = JSON.stringify({ bytes, sections, proseParagraphs, sampleCount, warmupCount, userAgent, results }, null, 2);
    result.dataset.state = "done";
  } catch (error) {
    result.textContent = String(error);
    result.dataset.state = "error";
  } finally {
    button.disabled = false;
  }
}

let demo: EditorView | undefined;
const start = () => {
  demo?.destroy();
  demo = undefined;
  void benchmark(root, run, output).finally(() => { demo = mount(root, true); });
};
run.addEventListener("click", start);
if (options.has("run")) start();
else demo = mount(root, true);
