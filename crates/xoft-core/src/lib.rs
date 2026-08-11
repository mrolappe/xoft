//! xoft core library.
//!
//! Design rule (see docs/plan.md): this crate performs no I/O and renders no text.
//! All output paths (CLI, testbed, tests) consume structured data from it.

pub mod codec;
pub mod corpus;
pub mod diagnostic;
pub mod grammar;
pub mod serialize;
pub mod strip_comments;
