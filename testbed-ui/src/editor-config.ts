import type { editor } from "monaco-editor";

// `createDiffEditor` defaults `originalEditable` to `false`; without this, the documented
// "type directly into the source pane" workflow silently rejects all keyboard input (M6.3
// round-41 finding #4) -- only the corpus-picker's `setValue()` path bypasses the read-only gate.
//
// `semanticHighlighting.enabled` isn't part of `IDiffEditorConstructionOptions`'s TS type (only
// `IStandaloneEditorConstructionOptions` extends the `IGlobalEditorOptions` that declares it), but
// Monaco does read it from the diff editor's own construction options at runtime -- and it must be
// set here, at construction time: `DocumentSemanticTokensFeature`'s one-time per-model scan (which
// gates whether a model's semantic tokens are ever fetched at all) runs synchronously during
// `createDiffEditor`, before any later `.updateOptions()` call can take effect (verified empirically
// against a real headless run -- a same-named `updateOptions()` call on each constituent editor
// after `setModel` was too late and left semantic tokens silently never fetched, which was M6.3
// round-41 finding #1's root cause).
export const DIFF_EDITOR_OPTIONS: editor.IDiffEditorConstructionOptions & editor.IGlobalEditorOptions = {
  automaticLayout: true,
  originalEditable: true,
  "semanticHighlighting.enabled": true,
};
