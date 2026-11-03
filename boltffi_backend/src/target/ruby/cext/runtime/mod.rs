//! Handwritten C support code that ships with every generated extension.
//!
//! `boltffi_ruby.h` is not a template. Each extension includes it as written.

use crate::{
    core::{FilePath, GeneratedFile, Result},
    target::ruby::name_style::extension_directory,
};

/// The runtime header that every generated extension includes.
const HEADER: &str = include_str!("../../../../../templates/target/ruby/runtime/boltffi_ruby.h");

/// Returns the runtime header beside the extension sources of `gem`.
pub fn header(gem: &str) -> Result<GeneratedFile> {
    Ok(GeneratedFile::new(
        FilePath::new(format!("{}/boltffi_ruby.h", extension_directory(gem)))?,
        HEADER,
    ))
}
