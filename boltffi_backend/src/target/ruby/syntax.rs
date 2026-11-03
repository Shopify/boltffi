use std::fmt;

use crate::{
    bridge::c,
    core::{Error, LanguageSyntax, Result, syntax::sealed},
};

/// Ruby syntax fragment family.
///
/// The Ruby target emits a C extension, so its types, expressions,
/// statements, and argument lists are the C bridge fragments. The Ruby-facing
/// names (method names, `Data` member names, and constant names) and the
/// literals in the Ruby package files use the fragments below. Each fragment
/// gets its constructor with the renderer that first builds it.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Syntax;

/// A Ruby method or `Data` member name, such as `echo_string`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier(String);

/// A Ruby constant name, such as `Point`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Constant(String);

/// A Ruby constant path, such as `MyLib::Native`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConstantPath(Vec<Constant>);

/// A double-quoted Ruby string literal.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Literal(String);

/// Methods every Ruby object answers.
///
/// The list is `Object`, `Kernel`, and `BasicObject` public instance methods
/// whose names Rust can spell, plus the private `initialize` family and the
/// private hooks that Ruby calls on any object, such as
/// `singleton_method_added`. Ruby keywords are not in the list: `Demo.end` is
/// a valid call.
const OBJECT_METHODS: &[&str] = &[
    "__id__",
    "__send__",
    "class",
    "clone",
    "define_singleton_method",
    "display",
    "dup",
    "enum_for",
    "extend",
    "freeze",
    "hash",
    "initialize",
    "initialize_clone",
    "initialize_copy",
    "initialize_dup",
    "inspect",
    "instance_eval",
    "instance_exec",
    "instance_variable_get",
    "instance_variable_set",
    "instance_variables",
    "itself",
    "method",
    "method_missing",
    "methods",
    "object_id",
    "private_methods",
    "protected_methods",
    "public_method",
    "public_methods",
    "public_send",
    "remove_instance_variable",
    "send",
    "singleton_class",
    "singleton_method",
    "singleton_method_added",
    "singleton_method_removed",
    "singleton_method_undefined",
    "singleton_methods",
    "tap",
    "then",
    "to_enum",
    "to_s",
    "yield_self",
];

/// Methods every Ruby module answers, in addition to [`OBJECT_METHODS`].
///
/// A module function with one of these names would replace the method on the
/// generated module: a function named `name` breaks `Module#name`, and one
/// named `private` breaks `private` inside a reopened `module` body. The list
/// is `Module` public instance methods plus the private visibility methods and
/// hooks a module body calls without a receiver.
const MODULE_METHODS: &[&str] = &[
    "alias_method",
    "ancestors",
    "append_features",
    "attr",
    "attr_accessor",
    "attr_reader",
    "attr_writer",
    "autoload",
    "class_eval",
    "class_exec",
    "class_variable_get",
    "class_variable_set",
    "class_variables",
    "const_added",
    "const_get",
    "const_missing",
    "const_set",
    "const_source_location",
    "constants",
    "define_method",
    "deprecate_constant",
    "extend_object",
    "extended",
    "include",
    "included",
    "included_modules",
    "instance_method",
    "instance_methods",
    "method_added",
    "method_removed",
    "method_undefined",
    "module_eval",
    "module_exec",
    "module_function",
    "name",
    "prepend",
    "prepend_features",
    "prepended",
    "private",
    "private_class_method",
    "private_constant",
    "private_instance_methods",
    "protected",
    "protected_instance_methods",
    "public",
    "public_class_method",
    "public_constant",
    "public_instance_method",
    "public_instance_methods",
    "refine",
    "refinements",
    "remove_class_variable",
    "remove_const",
    "remove_method",
    "set_temporary_name",
    "undef_method",
    "undefined_instance_methods",
    "using",
];

impl LanguageSyntax for Syntax {
    // The table lists the keyword tokens from Prism 1.9, the Ruby 4.1 grammar.
    // The `ruby-prism` Rust API exposes only `parse`, not a keyword table.
    // A parse per candidate name also adds a `bindgen` build step for all users.
    // `__END__` is not a keyword, but Ruby treats the marker as end of source.
    // The table rejects `__END__` as an identifier name for that reason.
    // Ruby adds keywords rarely, because a new keyword breaks valid programs.
    // Update this table when BoltFFI raises the supported Ruby version.
    const KEYWORDS: &'static [&'static str] = &[
        "BEGIN",
        "END",
        "__ENCODING__",
        "__END__",
        "__FILE__",
        "__LINE__",
        "alias",
        "and",
        "begin",
        "break",
        "case",
        "class",
        "def",
        "defined?",
        "do",
        "else",
        "elsif",
        "end",
        "ensure",
        "false",
        "for",
        "if",
        "in",
        "module",
        "next",
        "nil",
        "not",
        "or",
        "redo",
        "rescue",
        "retry",
        "return",
        "self",
        "super",
        "then",
        "true",
        "undef",
        "unless",
        "until",
        "when",
        "while",
        "yield",
    ];

    type Identifier = Identifier;
    type Type = c::TypeFragment;
    type Expr = c::Expression;
    type Stmt = c::Statement;
    type Literal = Literal;
    type Arguments = c::ArgumentList;
}

impl sealed::LanguageSyntax for Syntax {}

impl Identifier {
    /// Parses a module function name.
    ///
    /// A name that every Ruby module or object already answers gets a trailing
    /// underscore, so a Rust function named `name` becomes `Demo.name_`.
    pub fn module_function(identifier: impl Into<String>) -> Result<Self> {
        let identifier = identifier.into();
        let mut characters = identifier.chars();
        let valid = characters
            .next()
            .is_some_and(|first| first == '_' || first.is_ascii_lowercase())
            && characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
        if !valid {
            return Err(Error::InvalidRubyIdentifier { identifier });
        }
        let name = identifier.as_str();
        if OBJECT_METHODS.contains(&name) || MODULE_METHODS.contains(&name) {
            return Ok(Self(format!("{identifier}_")));
        }
        Ok(Self(identifier))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Constant {
    /// Parses a Ruby constant name made of ASCII word characters.
    /// Rejects keywords because the constant can start a Ruby expression.
    pub fn parse(constant: impl Into<String>) -> Result<Self> {
        let constant = constant.into();
        let mut characters = constant.chars();
        let valid = characters
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
            && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
            && !Syntax::keyword(&constant);
        if valid {
            Ok(Self(constant))
        } else {
            Err(Error::InvalidRubyIdentifier {
                identifier: constant,
            })
        }
    }

    /// Returns the constant text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ConstantPath {
    /// Parses a `::`-separated Ruby constant path.
    pub fn parse(path: &str) -> Result<Self> {
        path.split("::")
            .map(Constant::parse)
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }

    /// Creates a path with one constant.
    pub fn single(constant: Constant) -> Self {
        Self(vec![constant])
    }

    /// Returns the constants from the outermost to the innermost.
    pub fn segments(&self) -> &[Constant] {
        &self.0
    }
}

impl Literal {
    /// Creates a double-quoted Ruby string literal.
    ///
    /// Control characters become escapes, so the generated source stays a
    /// plain text file and line-ending conversion cannot change the value.
    pub fn string(value: &str) -> Self {
        let escaped = value.chars().fold(String::new(), |mut literal, character| {
            match character {
                '"' => literal.push_str("\\\""),
                '\\' => literal.push_str("\\\\"),
                '#' => literal.push_str("\\#"),
                '\n' => literal.push_str("\\n"),
                '\r' => literal.push_str("\\r"),
                '\t' => literal.push_str("\\t"),
                control if control.is_control() => {
                    literal.push_str(&format!("\\u{:04X}", u32::from(control)));
                }
                _ => literal.push(character),
            }
            literal
        });
        Self(format!("\"{escaped}\""))
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for ConstantPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self
            .0
            .iter()
            .map(Constant::as_str)
            .collect::<Vec<_>>()
            .join("::");
        formatter.write_str(&path)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl sealed::SyntaxFragment for Identifier {}
impl sealed::SyntaxFragment for Literal {}

#[cfg(test)]
mod tests {
    use crate::core::LanguageSyntax;

    use super::{Constant, ConstantPath, Identifier, Literal, Syntax};

    #[test]
    fn module_functions_that_shadow_core_methods_get_a_trailing_underscore() {
        for (rust, ruby) in [
            ("name", "name_"),
            ("hash", "hash_"),
            ("private", "private_"),
            // Ruby calls this hook while `Init_` defines each module function.
            ("singleton_method_added", "singleton_method_added_"),
            ("end", "end"),
            ("echo_string", "echo_string"),
            ("name_", "name_"),
        ] {
            assert_eq!(
                Identifier::module_function(rust).unwrap().as_str(),
                ruby,
                "{rust}"
            );
        }
    }

    #[test]
    fn module_functions_reject_invalid_ruby_names() {
        for invalid in ["", "Point", "has-dash", "1st", "é"] {
            assert!(
                Identifier::module_function(invalid).is_err(),
                "accepted {invalid}"
            );
        }
    }

    #[test]
    fn keyword_predicate_rejects_keywords_and_source_markers() {
        for reserved in [
            "class",
            "defined?",
            "nil",
            "self",
            "__FILE__",
            "__LINE__",
            "__ENCODING__",
            "__END__",
        ] {
            assert!(Syntax::keyword(reserved), "accepted {reserved}");
        }
        for ordinary in ["class?", "defined", "Nil", "begin_", "Point", "_1", "it"] {
            assert!(!Syntax::keyword(ordinary), "reserved {ordinary}");
        }
    }

    #[test]
    fn keyword_predicate_does_not_classify_other_source_as_a_keyword() {
        for source in ["", "class Point; end", "1", "Point::Native", "name + other"] {
            assert!(!Syntax::keyword(source), "reserved {source}");
        }
    }

    #[test]
    fn constants_reject_invalid_ruby_names() {
        assert!(Constant::parse("point").is_err());
        assert!(Constant::parse("My-Lib").is_err());
        assert!(ConstantPath::parse("MyLib::").is_err());
        assert_eq!(
            ConstantPath::parse("MyLib::Native").unwrap().to_string(),
            "MyLib::Native"
        );
    }

    #[test]
    fn constants_reject_ruby_keywords() {
        for keyword in ["BEGIN", "END"] {
            assert!(Constant::parse(keyword).is_err(), "accepted {keyword}");
            assert!(ConstantPath::parse(&format!("{keyword}::Native")).is_err());
        }
        assert_eq!(Constant::parse("Begin").unwrap().as_str(), "Begin");
        assert_eq!(Constant::parse("End").unwrap().as_str(), "End");
    }

    #[test]
    fn string_literals_escape_interpolation() {
        assert_eq!(Literal::string("a#{b}\"c").to_string(), "\"a\\#{b}\\\"c\"");
    }

    #[test]
    fn string_literals_escape_control_characters() {
        assert_eq!(
            Literal::string("\0\r\t\u{1f}\u{7f}\u{85}1é").to_string(),
            "\"\\u0000\\r\\t\\u001F\\u007F\\u00851é\""
        );
        assert!(!Literal::string("\r\n").to_string().contains('\r'));
    }
}
