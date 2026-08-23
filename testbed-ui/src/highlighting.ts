// M6.3 -- semantic-token highlighting for both grammars, driven by web-tree-sitter parsing
// in the browser plus the grammars' own `queries/highlights.scm` (vendored into
// `src/grammars/`, see `scripts/build-wasm-grammars.sh`; both grammars currently ship the
// same query text, one copy is enough). A Monaco `languages.SemanticTokensProvider`, per
// docs/plan.md's explicit "semantic-tokens provider, not Monarch" note -- not a second,
// hand-rolled tokenizer.
import * as monaco from "monaco-editor";
import { Language, Node as TSNode, Parser, Query } from "web-tree-sitter";

import treeSitterWasmUrl from "web-tree-sitter/web-tree-sitter.wasm?url";
import highlightsQueryText from "./grammars/highlights.scm?raw";
import oberonXWasmUrl from "./grammars/oberon-x.wasm?url";
import oberon2WasmUrl from "./grammars/oberon2.wasm?url";

export type Dialect = "oberon2" | "oberon-x";

const WASM_URL: Record<Dialect, string> = {
  oberon2: oberon2WasmUrl,
  "oberon-x": oberonXWasmUrl,
};

// Monaco doesn't require standard LSP token-type names, but using them lets the built-in
// themes color tokens with no extra theme rules of our own.
const TOKEN_TYPES = [
  "namespace",
  "type",
  "property",
  "variable",
  "function",
  "keyword",
  "operator",
  "string",
  "number",
  "comment",
] as const;
const TOKEN_MODIFIERS = ["definition", "defaultLibrary", "readonly"] as const;

type TokenType = (typeof TOKEN_TYPES)[number];
type TokenModifier = (typeof TOKEN_MODIFIERS)[number];

const LEGEND: monaco.languages.SemanticTokensLegend = {
  tokenTypes: [...TOKEN_TYPES],
  tokenModifiers: [...TOKEN_MODIFIERS],
};

// One entry per capture name in highlights.scm. A capture not listed here (e.g. `error`,
// left to the diagnostics-driven markers instead) gets no semantic token -- the leaf just
// keeps the editor's default foreground.
const CAPTURE_TOKENS: Partial<Record<string, { type: TokenType; mods?: TokenModifier[] }>> = {
  namespace: { type: "namespace" },
  constant: { type: "variable", mods: ["readonly"] },
  "type.definition": { type: "type", mods: ["definition"] },
  property: { type: "property" },
  variable: { type: "variable" },
  type: { type: "type" },
  "type.builtin": { type: "type", mods: ["defaultLibrary"] },
  "function.call": { type: "function" },
  "function.builtin": { type: "function", mods: ["defaultLibrary"] },
  function: { type: "function", mods: ["definition"] },
  include: { type: "keyword" },
  "keyword.return": { type: "keyword" },
  conditional: { type: "keyword" },
  repeat: { type: "keyword" },
  keyword: { type: "keyword" },
  "keyword.operator": { type: "operator" },
  operator: { type: "operator" },
  "punctuation.bracket": { type: "operator" },
  "punctuation.delimiter": { type: "operator" },
  "constant.builtin": { type: "keyword" },
  boolean: { type: "keyword" },
  number: { type: "number" },
  string: { type: "string" },
  comment: { type: "comment" },
};

function modifierBits(mods: TokenModifier[] | undefined): number {
  if (!mods) return 0;
  return mods.reduce((bits, m) => bits | (1 << TOKEN_MODIFIERS.indexOf(m)), 0);
}

// This monaco-editor version's typings don't export the `languages.SemanticTokensBuilder`
// helper class, so the delta encoding (LSP semantic-tokens spec: 5 uint32s per token --
// deltaLine, deltaStartChar-or-absolute, length, tokenType, tokenModifiers) is done by hand.
// Tokens must be pushed in ascending (line, char) order -- true here since `computeTokens`
// walks leaves left to right.
class SemanticTokensBuilder {
  private data: number[] = [];
  private prevLine = 0;
  private prevChar = 0;

  push(line: number, char: number, length: number, tokenType: number, tokenModifiers: number) {
    const deltaLine = line - this.prevLine;
    const deltaChar = deltaLine === 0 ? char - this.prevChar : char;
    this.data.push(deltaLine, deltaChar, length, tokenType, tokenModifiers);
    this.prevLine = line;
    this.prevChar = char;
  }

  build(): monaco.languages.SemanticTokens {
    return { data: new Uint32Array(this.data) };
  }
}

let initPromise: Promise<void> | null = null;
const parsers = new Map<Dialect, Parser>();
const queries = new Map<Dialect, Query>();

function ensureInit(): Promise<void> {
  initPromise ??= Parser.init({ locateFile: () => treeSitterWasmUrl });
  return initPromise;
}

async function ensureDialect(dialect: Dialect): Promise<{ parser: Parser; query: Query }> {
  await ensureInit();
  let parser = parsers.get(dialect);
  if (!parser) {
    const language = await Language.load(WASM_URL[dialect]);
    parser = new Parser();
    parser.setLanguage(language);
    parsers.set(dialect, parser);
    queries.set(dialect, new Query(language, highlightsQueryText));
  }
  return { parser, query: queries.get(dialect)! };
}

/// Byte offset (tree-sitter's `Node.startIndex`, UTF-8) -> UTF-16 code-unit offset (what
/// Monaco's `model.getPositionAt` expects). Codepoint boundaries only -- every tree-sitter
/// node boundary lands on one, so a direct offset lookup (no interpolation) is exact.
function byteToCharMap(text: string): Map<number, number> {
  const map = new Map<number, number>();
  const encoder = new TextEncoder();
  let byte = 0;
  let char = 0;
  for (const cp of text) {
    map.set(byte, char);
    byte += encoder.encode(cp).length;
    char += cp.length;
  }
  map.set(byte, char);
  return map;
}

function collectLeaves(node: TSNode, out: TSNode[]): void {
  if (node.childCount === 0) {
    out.push(node);
    return;
  }
  for (let i = 0; i < node.childCount; i++) {
    const child = node.child(i);
    if (child) collectLeaves(child, out);
  }
}

function pushToken(
  builder: SemanticTokensBuilder,
  model: monaco.editor.ITextModel,
  startChar: number,
  endChar: number,
  type: TokenType,
  mods: TokenModifier[] | undefined,
) {
  const typeIdx = TOKEN_TYPES.indexOf(type);
  const bits = modifierBits(mods);
  const start = model.getPositionAt(startChar);
  const end = model.getPositionAt(endChar);
  if (start.lineNumber === end.lineNumber) {
    builder.push(start.lineNumber - 1, start.column - 1, end.column - start.column, typeIdx, bits);
    return;
  }
  // A leaf spanning multiple lines (block comments/pragmas): one push per line, since
  // Monaco's semantic tokens are single-line by construction.
  builder.push(
    start.lineNumber - 1,
    start.column - 1,
    model.getLineMaxColumn(start.lineNumber) - start.column,
    typeIdx,
    bits,
  );
  for (let line = start.lineNumber + 1; line < end.lineNumber; line++) {
    builder.push(line - 1, 0, model.getLineMaxColumn(line) - 1, typeIdx, bits);
  }
  builder.push(end.lineNumber - 1, 0, end.column - 1, typeIdx, bits);
}

async function computeTokens(
  model: monaco.editor.ITextModel,
  dialect: Dialect,
): Promise<monaco.languages.SemanticTokens> {
  const text = model.getValue();
  const { parser, query } = await ensureDialect(dialect);
  const tree = parser.parse(text);
  const builder = new SemanticTokensBuilder();
  if (!tree) return builder.build();

  // Innermost capture wins: map each captured node's id to its capture name, then for every
  // leaf walk up to the nearest captured ancestor-or-self. This keeps emitted tokens
  // non-overlapping (Monaco requires it) without hand-rolling tree-sitter's own
  // highlight-priority/splitting algorithm.
  const captureByNodeId = new Map<number, string>();
  for (const c of query.captures(tree.rootNode)) {
    captureByNodeId.set(c.node.id, c.name);
  }

  const leaves: TSNode[] = [];
  collectLeaves(tree.rootNode, leaves);

  const byteToChar = byteToCharMap(text);
  for (const leaf of leaves) {
    let node: TSNode | null = leaf;
    let captureName: string | undefined;
    while (node) {
      captureName = captureByNodeId.get(node.id);
      if (captureName) break;
      node = node.parent;
    }
    const spec = captureName ? CAPTURE_TOKENS[captureName] : undefined;
    if (!spec) continue;

    const startChar = byteToChar.get(leaf.startIndex);
    const endChar = byteToChar.get(leaf.endIndex);
    if (startChar === undefined || endChar === undefined || startChar === endChar) continue;
    pushToken(builder, model, startChar, endChar, spec.type, spec.mods);
  }

  return builder.build();
}

export function registerHighlighting(): void {
  for (const dialect of ["oberon2", "oberon-x"] as const) {
    monaco.languages.register({ id: dialect });
    monaco.languages.registerDocumentSemanticTokensProvider(dialect, {
      getLegend: () => LEGEND,
      provideDocumentSemanticTokens: (model) => computeTokens(model, dialect),
      releaseDocumentSemanticTokens: () => {},
    });
  }
}
