//! Byte offset -> 1-based line/column, for rendering a `Diagnostic`'s byte span in an editor
//! (M6.3, docs/plan.md). The offset indexes into the same text tree-sitter parsed --
//! `Document`'s codec text (D3), where each original byte is exactly one `char` -- so
//! counting `char`s already counts UTF-16 code units 1:1, which is what Monaco's `column`
//! expects; a naive UTF-8 byte count would overcount any byte above 0x7F.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

pub fn byte_to_position(text: &str, byte: usize) -> Position {
    let mut line = 1u32;
    let mut column = 1u32;
    for (i, c) in text.char_indices() {
        if i >= byte {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Position { line, column }
}
