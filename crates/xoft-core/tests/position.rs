//! M6.3 -- byte offset -> 1-based line/column, against `Document`'s codec text (D3), not the
//! original file's raw bytes (docs/plan.md, NEXT.md history round 39/40).

use xoft_core::position::{byte_to_position, Position};

fn pos(line: u32, column: u32) -> Position {
    Position { line, column }
}

#[test]
fn byte_zero_is_line_one_column_one() {
    assert_eq!(byte_to_position("abc", 0), pos(1, 1));
}

#[test]
fn counts_columns_within_the_first_line() {
    assert_eq!(byte_to_position("abc", 2), pos(1, 3));
}

#[test]
fn a_newline_starts_a_new_line_and_resets_the_column() {
    let text = "ab\ncd";
    assert_eq!(byte_to_position(text, 3), pos(2, 1));
    assert_eq!(byte_to_position(text, 4), pos(2, 2));
}

#[test]
fn counts_multiple_newlines() {
    let text = "a\nb\nc";
    assert_eq!(byte_to_position(text, 4), pos(3, 1));
}

#[test]
fn end_of_text_offset_is_valid() {
    let text = "ab\ncd";
    assert_eq!(byte_to_position(text, text.len()), pos(2, 3));
}

#[test]
fn a_high_byte_char_counts_as_one_column_despite_being_two_utf8_bytes() {
    // Document::from_bytes maps byte 0xE9 to char U+00E9 ('\u{e9}'), which is 2 bytes in
    // this Rust String's own UTF-8 encoding -- the column count must still advance by one
    // per original byte/char, not by UTF-8 byte width, or Monaco (UTF-16-unit columns)
    // would misalign on any non-ASCII source byte.
    let text = "a\u{e9}c";
    assert_eq!(byte_to_position(text, text.len()), pos(1, 4));
}
