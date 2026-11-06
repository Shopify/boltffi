//! Encoded bytes from Rust into Ruby objects.
//!
//! Each leaf renders a C expression that reads one `VALUE` from `reader`.
//! Only strings and bytes move today; other shapes report unsupported.

use boltffi_binding::{
    BuiltinType, CallbackId, ClassId, CodecRead, CustomTypeId, ElementCount, EnumId, MapKind, Op,
    Primitive, ReadPlan, RecordId,
};

use crate::{
    bridge::c::{Expression, Identifier, Statement},
    core::Result,
    target::ruby::cext::support::unsupported,
};

/// Renders decode statements for codec trees.
pub struct Reader;

impl Reader {
    /// Renders a statement that decodes `plan` into `destination`.
    pub fn decode(plan: &ReadPlan, destination: &str) -> Result<Statement> {
        let value = plan.render_with(&mut Reader)?;
        Ok(Statement::new(format!("{destination} = {value};")))
    }

    fn call(function: &str) -> Result<Expression> {
        let function = Identifier::parse(function)?;
        Ok(Expression::new(format!("{function}(reader)")))
    }
}

impl CodecRead for Reader {
    type Expr = Result<Expression>;

    fn primitive(&mut self, _: Primitive) -> Self::Expr {
        unsupported("encoded primitive")
    }

    fn string(&mut self) -> Self::Expr {
        Self::call("boltffi_ruby_read_string")
    }

    fn utf8_string(&mut self) -> Self::Expr {
        unsupported("unframed UTF-8 string")
    }

    fn raw_bytes(&mut self) -> Self::Expr {
        unsupported("unframed byte buffer")
    }

    fn interned_string(&mut self, _: &[String]) -> Self::Expr {
        unsupported("interned string")
    }

    fn bytes(&mut self) -> Self::Expr {
        Self::call("boltffi_ruby_read_binary")
    }

    fn direct_record(&mut self, _: RecordId) -> Self::Expr {
        unsupported("record")
    }

    fn encoded_record(&mut self, _: RecordId) -> Self::Expr {
        unsupported("record")
    }

    fn c_style_enum(&mut self, _: EnumId) -> Self::Expr {
        unsupported("enum")
    }

    fn data_enum(&mut self, _: EnumId) -> Self::Expr {
        unsupported("enum")
    }

    fn class_handle(&mut self, _: ClassId) -> Self::Expr {
        unsupported("class handle")
    }

    fn callback_handle(&mut self, _: CallbackId) -> Self::Expr {
        unsupported("callback handle")
    }

    fn custom(&mut self, _: CustomTypeId, _: Self::Expr) -> Self::Expr {
        unsupported("custom type")
    }

    fn builtin(&mut self, _: BuiltinType) -> Self::Expr {
        unsupported("builtin type")
    }

    fn optional(&mut self, _: Self::Expr) -> Self::Expr {
        unsupported("optional value")
    }

    fn sequence(&mut self, _: &Op<ElementCount>, _: Self::Expr) -> Self::Expr {
        unsupported("sequence")
    }

    fn tuple(&mut self, _: Vec<Self::Expr>) -> Self::Expr {
        unsupported("tuple")
    }

    fn result(&mut self, _: Self::Expr, _: Self::Expr) -> Self::Expr {
        unsupported("Result value")
    }

    fn map(&mut self, _: MapKind, _: Self::Expr, _: Self::Expr) -> Self::Expr {
        unsupported("map")
    }
}
