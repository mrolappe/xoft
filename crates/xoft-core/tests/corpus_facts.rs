//! M0.2 — classification of corpus files. Written before the implementation (TDD).

use xoft_core::corpus::{EncodingClass, FileFacts, LineEndings};

#[test]
fn plain_ascii_lf() {
    let f = FileFacts::classify(b"MODULE M;\nEND M.\n");
    assert_eq!(f.line_endings, LineEndings::Lf);
    assert_eq!(f.encoding, EncodingClass::Utf8);
    assert!(!f.has_tabs);
    assert_eq!(f.bytes, 17);
}

#[test]
fn crlf_is_detected_and_not_mistaken_for_cr() {
    let f = FileFacts::classify(b"MODULE M;\r\nEND M.\r\n");
    assert_eq!(f.line_endings, LineEndings::Crlf);
}

#[test]
fn bare_cr_is_its_own_class() {
    let f = FileFacts::classify(b"MODULE M;\rEND M.\r");
    assert_eq!(f.line_endings, LineEndings::Cr);
}

#[test]
fn mixed_line_endings_are_reported_as_mixed() {
    let f = FileFacts::classify(b"a\r\nb\nc\n");
    assert_eq!(f.line_endings, LineEndings::Mixed);
}

#[test]
fn no_line_break_at_all() {
    let f = FileFacts::classify(b"MODULE M; END M.");
    assert_eq!(f.line_endings, LineEndings::None);
}

#[test]
fn latin1_umlaut_is_high_bytes_not_utf8() {
    // 0xFC is 'ü' in ISO-8859-1 and is not valid standalone UTF-8.
    let f = FileFacts::classify(b"(* Gr\xfc\xdfe *)\n");
    assert_eq!(f.encoding, EncodingClass::HighBytes);
}

#[test]
fn valid_utf8_multibyte_is_utf8() {
    let f = FileFacts::classify("(* Grüße *)\n".as_bytes());
    assert_eq!(f.encoding, EncodingClass::Utf8);
}

#[test]
fn tabs_are_flagged() {
    assert!(FileFacts::classify(b"\tBEGIN\n").has_tabs);
}

#[test]
fn sha256_is_lowercase_hex_of_the_exact_bytes() {
    // Known vector: sha256("") — proves we hash the raw bytes, nothing normalized.
    let f = FileFacts::classify(b"");
    assert_eq!(
        f.sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(f.bytes, 0);
    assert_eq!(f.line_endings, LineEndings::None);
}
