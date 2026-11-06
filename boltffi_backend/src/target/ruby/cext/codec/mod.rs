//! Encoded values between Ruby objects and BoltFFI wire bytes.
//!
//! `read` decodes what Rust returns and `write` encodes what Ruby passes.
//! Both feed the shared codec walkers, so this module only spells the
//! leaves. Ported from Max VelDink's `ruby-lean-target` draft.

pub mod read;
pub mod write;
