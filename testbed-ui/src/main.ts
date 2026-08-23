import * as monaco from "monaco-editor";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

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

// One DiffEditor doubles as the source editor: its original model is the live, editable
// source (typed, or loaded from the corpus picker); its modified model holds transpile's
// output, refreshed on demand rather than running a separate editor alongside the diff view.
const originalModel = monaco.editor.createModel(sampleSource, undefined);
const modifiedModel = monaco.editor.createModel("", undefined);

const diffEditor = monaco.editor.createDiffEditor(diffContainer, {
  automaticLayout: true,
});
diffEditor.setModel({ original: originalModel, modified: modifiedModel });
diffEditor.getModifiedEditor().updateOptions({ readOnly: true });

function renderDiagnostics(diagnostics: Diagnostic[]) {
  diagnosticsEl.innerHTML = "";
  for (const d of diagnostics) {
    const li = document.createElement("li");
    li.textContent = `${d.start_byte}-${d.end_byte}: ${d.message}`;
    diagnosticsEl.appendChild(li);
  }
}

async function runTranspile() {
  const direction = directionEl.value as Direction;
  const text = originalModel.getValue();
  try {
    const result = await invoke<TranspileResult>("transpile", { direction, text });
    modifiedModel.setValue(result.output);
    renderDiagnostics(result.diagnostics);
  } catch (e) {
    renderDiagnostics([{ start_byte: 0, end_byte: 0, message: String(e) }]);
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
  } catch (e) {
    renderDiagnostics([{ start_byte: 0, end_byte: 0, message: String(e) }]);
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
