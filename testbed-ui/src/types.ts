// Mirrors the Rust command types in crates/xoft-testbed/src/commands.rs and
// crates/xoft-cli/src/manifest.rs. Command *arguments* are camelCase (Tauri's IPC
// auto-conversion); every returned struct keeps its own `#[derive(Serialize)]` casing, which
// here is snake_case (plain derive, no rename_all) -- confirmed empirically in M6.1, not
// assumed.

export type Direction = "oberon-x-to-oberon2" | "oberon2-to-oberon-x";

export interface FileFacts {
  bytes: number;
  sha256: string;
  line_endings: string;
  encoding: string;
  has_tabs: boolean;
}

export type Entry = FileFacts & {
  root: string;
  path: string;
};

export interface RootSummary {
  alias: string;
  origin: string;
  license: string;
  files: number;
  bytes: number;
}

export interface Manifest {
  roots: RootSummary[];
  files: Entry[];
}

export interface Position {
  line: number;
  column: number;
}

export interface Diagnostic {
  start: Position;
  end: Position;
  message: string;
}

export interface TranspileResult {
  output: string;
  diagnostics: Diagnostic[];
}
