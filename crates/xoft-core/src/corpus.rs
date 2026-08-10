//! Facts about a corpus file, derived from its raw bytes.
//!
//! Pure: takes bytes, returns data. The walking and writing live in the CLI.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineEndings {
    None,
    Lf,
    Crlf,
    Cr,
    Mixed,
}

/// Only the distinction the core cares about: whether the file is plain UTF-8,
/// or carries bytes >= 0x80 that some single-byte charset gives meaning to.
/// Which charset that is, is a display concern (see D3 in docs/plan.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EncodingClass {
    Utf8,
    HighBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFacts {
    pub bytes: usize,
    pub sha256: String,
    pub line_endings: LineEndings,
    pub encoding: EncodingClass,
    pub has_tabs: bool,
}

impl FileFacts {
    pub fn classify(bytes: &[u8]) -> Self {
        FileFacts {
            bytes: bytes.len(),
            sha256: hex(&Sha256::digest(bytes)),
            line_endings: line_endings(bytes),
            encoding: match std::str::from_utf8(bytes) {
                Ok(_) => EncodingClass::Utf8,
                Err(_) => EncodingClass::HighBytes,
            },
            has_tabs: bytes.contains(&b'\t'),
        }
    }
}

fn line_endings(bytes: &[u8]) -> LineEndings {
    let (mut lf, mut crlf, mut cr) = (0u32, 0u32, 0u32);
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                crlf += 1;
                i += 1;
            }
            b'\r' => cr += 1,
            b'\n' => lf += 1,
            _ => {}
        }
        i += 1;
    }
    match (lf > 0, crlf > 0, cr > 0) {
        (false, false, false) => LineEndings::None,
        (true, false, false) => LineEndings::Lf,
        (false, true, false) => LineEndings::Crlf,
        (false, false, true) => LineEndings::Cr,
        _ => LineEndings::Mixed,
    }
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
