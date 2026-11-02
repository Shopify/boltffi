//! The extension source file: function definitions and the `Init_` entry
//! point that defines them.

use askama::Template;
use boltffi_binding::{DeclarationRef, Native};

use crate::{
    bridge::c::{Identifier, Literal},
    core::{GeneratedFile, RenderedDeclaration, Result},
    target::ruby::{
        name_style::{NameScope, extension_directory, extension_stem},
        syntax::ConstantPath,
    },
};

use super::function::Registration;

#[derive(Template)]
#[template(path = "target/ruby/extension.c", escape = "none")]
struct ExtensionTemplate {
    init: Identifier,
    modules: Vec<Literal>,
    definitions: Vec<String>,
    functions: Vec<ModuleFunction>,
}

/// One module function that `Init_` defines.
struct ModuleFunction {
    name: Literal,
    wrapper: Identifier,
    arity: i32,
}

/// Renders the extension source file, `<stem>.c`.
pub fn render(
    gem: &str,
    module: &ConstantPath,
    declarations: &[RenderedDeclaration<'_, Native>],
) -> Result<GeneratedFile> {
    let stem = extension_stem(gem);
    let mut methods = NameScope::new(format!("module `{module}`"));
    let mut definitions = Vec::new();
    let mut functions = Vec::new();
    for rendered in declarations {
        if let DeclarationRef::Function(declaration) = rendered.declaration() {
            let registration = Registration::from_declaration(declaration)?;
            methods.claim(registration.ruby_name.as_str(), registration.subject)?;
            functions.push(ModuleFunction {
                name: Literal::string(registration.ruby_name.as_str()),
                wrapper: registration.wrapper,
                arity: registration.arity,
            });
        }
        let primary = rendered.emitted().primary_chunk().as_str();
        if !primary.is_empty() {
            definitions.push(primary.to_owned());
        }
    }
    let source = ExtensionTemplate {
        init: Identifier::parse(format!("Init_{stem}"))?,
        modules: module
            .segments()
            .iter()
            .map(|constant| Literal::string(constant.as_str()))
            .collect(),
        definitions,
        functions,
    }
    .render()?;
    super::file(format!("{}/{stem}.c", extension_directory(gem)), source)
}
