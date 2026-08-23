import * as monaco from "monaco-editor";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

import type { Dialect } from "./highlighting";
import { registerHighlighting } from "./highlighting";
import type { Diagnostic, Direction, Entry, Manifest, TranspileResult } from "./types";
import "./style.css";

self.MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};

const { invoke } = window.__TAURI__.core;

const corpusListEl = document.getElementById("corpus-list") as HTMLUListElement;
const directionEl = document.getElementById("direction") as HTMLSelectElement;
const transpileButton = document.getElementById("transpile-run") as HTMLButtonElement;
const diagnosticsEl = document.getElementById("diagnostics") as HTMLUListElement;
const diffContainer = document.getElementById("diff-editor") as HTMLDivElement;

const sampleSource = "MODULE M;\nBEGIN\n  UNLESS x DO y := 1 END\nEND M.\n";

const DIAGNOSTIC_OWNER = "xoft";

registerHighlighting();

// One DiffEditor doubles as the source editor: its original model is the live, editable
// source (typed, or loaded from the corpus picker); its modified model holds transpile's
// output, refreshed on demand rather than running a separate editor alongside the diff view.
const originalModel = monaco.editor.createModel(sampleSource, "oberon-x");
const modifiedModel = monaco.editor.createModel("", "oberon2");

const diffEditor = monaco.editor.createDiffEditor(diffContainer, {
  automaticLayout: true,
});
diffEditor.setModel({ original: originalModel, modified: modifiedModel });
diffEditor.getModifiedEditor().updateOptions({ readOnly: true });

function dialectsForDirection(direction: Direction): { source: Dialect; output: Dialect } {
  return direction === "oberon-x-to-oberon2"
    ? { source: "oberon-x", output: "oberon2" }
    : { source: "oberon2", output: "oberon-x" };
}

function applyDialects() {
  const { source, output } = dialectsForDirection(directionEl.value as Direction);
  monaco.editor.setModelLanguage(originalModel, source);
  monaco.editor.setModelLanguage(modifiedModel, output);
}

applyDialects();
directionEl.addEventListener("change", applyDialects);

let currentDiagnostics: Diagnostic[] = [];
let diagnosticItems: HTMLLIElement[] = [];

function diagnosticRange(d: Diagnostic): monaco.Range {
  return new monaco.Range(d.start.line, d.start.column, d.end.line, d.end.column);
}

function selectDiagnosticItem(index: number) {
  diagnosticItems.forEach((li, i) => li.classList.toggle("selected", i === index));
  diagnosticItems[index]?.scrollIntoView({ block: "nearest" });
}

function jumpToDiagnostic(d: Diagnostic) {
  const editor = diffEditor.getOriginalEditor();
  const range = diagnosticRange(d);
  editor.revealRangeInCenter(range);
  editor.setSelection(range);
  editor.focus();
}

function renderDiagnostics(diagnostics: Diagnostic[]) {
  currentDiagnostics = diagnostics;
  diagnosticsEl.innerHTML = "";
  diagnosticItems = diagnostics.map((d, i) => {
    const li = document.createElement("li");
    li.textContent = `${d.start.line}:${d.start.column}-${d.end.line}:${d.end.column}: ${d.message}`;
    li.addEventListener("click", () => {
      selectDiagnosticItem(i);
      jumpToDiagnostic(d);
    });
    diagnosticsEl.appendChild(li);
    return li;
  });

  monaco.editor.setModelMarkers(
    originalModel,
    DIAGNOSTIC_OWNER,
    diagnostics.map((d) => ({
      severity: monaco.MarkerSeverity.Error,
      startLineNumber: d.start.line,
      startColumn: d.start.column,
      endLineNumber: d.end.line,
      endColumn: d.end.column,
      message: d.message,
    })),
  );
}

// Reverse navigation: clicking an ERROR/MISSING span (marked via the markers above) in the
// editor selects the matching diagnostic in the list.
diffEditor.getOriginalEditor().onMouseDown((e) => {
  const position = e.target.position;
  if (!position) return;
  const index = currentDiagnostics.findIndex((d) => diagnosticRange(d).containsPosition(position));
  if (index >= 0) selectDiagnosticItem(index);
});

async function runTranspile() {
  const direction = directionEl.value as Direction;
  const text = originalModel.getValue();
  try {
    const result = await invoke<TranspileResult>("transpile", { direction, text });
    modifiedModel.setValue(result.output);
    renderDiagnostics(result.diagnostics);
  } catch (e) {
    renderDiagnostics([{ start: { line: 1, column: 1 }, end: { line: 1, column: 1 }, message: String(e) }]);
  }
}

transpileButton.addEventListener("click", () => void runTranspile());

async function loadCorpusFile(entry: Entry) {
  try {
    const raw = await invoke<number[]>("read_corpus_file", {
      root: entry.root,
      path: entry.path,
    });
    originalModel.setValue(new TextDecoder().decode(new Uint8Array(raw)));
    renderDiagnostics([]);
  } catch (e) {
    renderDiagnostics([{ start: { line: 1, column: 1 }, end: { line: 1, column: 1 }, message: String(e) }]);
  }
}

function renderCorpusList(manifest: Manifest) {
  corpusListEl.innerHTML = "";
  let currentRoot = "";
  for (const entry of manifest.files) {
    if (entry.root !== currentRoot) {
      currentRoot = entry.root;
      const heading = document.createElement("li");
      heading.className = "corpus-root";
      heading.textContent = currentRoot;
      corpusListEl.appendChild(heading);
    }
    const li = document.createElement("li");
    li.className = "corpus-file";
    li.textContent = entry.path;
    li.addEventListener("click", () => void loadCorpusFile(entry));
    corpusListEl.appendChild(li);
  }
}

async function loadCorpus() {
  try {
    const manifest = await invoke<Manifest>("list_corpus");
    renderCorpusList(manifest);
  } catch (e) {
    corpusListEl.innerHTML = "";
    const li = document.createElement("li");
    li.className = "corpus-error";
    li.textContent = String(e);
    corpusListEl.appendChild(li);
  }
}

void loadCorpus();
