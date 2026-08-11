//! Byte <-> char codec (D3 in docs/plan.md).
//!
//! Each input byte maps to the Unicode codepoint of the same value (U+0000-U+00FF), so
//! parsing runs on an ordinary `String` with byte-identity guaranteed by construction for
//! any single-byte charset. Which charset the bytes actually mean is a display concern,
//! applied by the CLI/testbed and nowhere in this crate.

pub struct Document {
    pub text: String,
}

impl Document {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Document {
            text: bytes.iter().map(|&b| b as char).collect(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.text.chars().map(|c| c as u32 as u8).collect()
    }
}
