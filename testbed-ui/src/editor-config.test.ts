import { describe, expect, it } from "vitest";

import { DIFF_EDITOR_OPTIONS } from "./editor-config";

describe("DIFF_EDITOR_OPTIONS", () => {
  it("makes the source (original) pane editable", () => {
    expect(DIFF_EDITOR_OPTIONS.originalEditable).toBe(true);
  });

  it("keeps automatic layout enabled", () => {
    expect(DIFF_EDITOR_OPTIONS.automaticLayout).toBe(true);
  });

  it("forces semantic highlighting on at construction time, regardless of the active theme's own setting", () => {
    // Must be set here, not via a later updateOptions() call -- DocumentSemanticTokensFeature's
    // one-time per-model scan (which gates whether semantic tokens are ever fetched for a model)
    // runs synchronously during createDiffEditor, before any updateOptions() call can take effect.
    expect(DIFF_EDITOR_OPTIONS["semanticHighlighting.enabled"]).toBe(true);
  });
});
