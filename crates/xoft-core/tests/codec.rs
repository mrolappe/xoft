//! M2.1 — byte<->char codec (D3). Written before the implementation (TDD).

use xoft_core::codec::Document;

#[test]
fn round_trips_all_256_byte_values() {
    let bytes: Vec<u8> = (0..=255).collect();
    let doc = Document::from_bytes(&bytes);
    assert_eq!(doc.to_bytes(), bytes);
}

#[test]
fn round_trips_a_typical_source_file() {
    let bytes = b"MODULE M;\nEND M.\n";
    let doc = Document::from_bytes(bytes);
    assert_eq!(doc.to_bytes(), bytes.to_vec());
}

#[test]
fn each_byte_maps_to_exactly_one_char() {
    let bytes = [0x00, 0x41, 0x7F, 0x80, 0xFF];
    let doc = Document::from_bytes(&bytes);
    assert_eq!(doc.text.chars().count(), bytes.len());
}
