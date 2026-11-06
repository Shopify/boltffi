//! Ruby objects into encoded bytes for Rust.
//!
//! Each leaf renders a C statement that checks one Ruby value and appends
//! its bytes to `writer`. Only strings and bytes move today; other shapes
//! report unsupported.

use boltffi_binding::{
    BinderId, BuiltinType, CallbackId, ClassId, CodecWrite, CustomTypeId, ElementCount, EnumId,
    MapKind, Op, Primitive, RecordId, ValueRef, ValueRoot, WritePlan,
};

use crate::{
    bridge::c::{Expression, Identifier, Statement},
    core::Result,
    target::ruby::cext::support::unsupported,
};

/// Renders encode statements for codec trees.
pub struct Writer {
    root: Expression,
}

impl Writer {
    /// Creates a writer whose plans start at the C expression `root`.
    pub fn new(root: Expression) -> Self {
        Self { root }
    }

    /// Renders statements that append `plan` to `writer`.
    pub fn encode(&mut self, plan: &WritePlan) -> Result<Statement> {
        plan.render_with(self)
            .into_iter()
            .map(|statement| statement.map(|statement| statement.to_string()))
            .collect::<Result<Vec<_>>>()
            .map(|statements| Statement::new(statements.join("\n")))
    }

    /// Returns the C expression for the value. Leaves only read the root.
    fn reference(&self, value: &ValueRef) -> Result<Expression> {
        match value.root() {
            ValueRoot::SelfValue | ValueRoot::Named(_) | ValueRoot::Local(_)
                if value.path().is_empty() =>
            {
                Ok(self.root.clone())
            }
            _ => unsupported("codec value path"),
        }
    }

    fn call(&self, function: &str, value: &ValueRef) -> Vec<Result<Statement>> {
        vec![Identifier::parse(function).and_then(|function| {
            let value = self.reference(value)?;
            Ok(Statement::new(format!("{function}(writer, {value});")))
        })]
    }
}

impl CodecWrite for Writer {
    type Stmt = Result<Statement>;

    fn primitive(&mut self, _: Primitive, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("encoded primitive")]
    }

    fn string(&mut self, value: &ValueRef) -> Vec<Self::Stmt> {
        self.call("boltffi_ruby_write_string", value)
    }

    fn utf8_string(&mut self, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("unframed UTF-8 string")]
    }

    fn raw_bytes(&mut self, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("unframed byte buffer")]
    }

    fn interned_string(&mut self, _: &[String], _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("interned string")]
    }

    fn bytes(&mut self, value: &ValueRef) -> Vec<Self::Stmt> {
        self.call("boltffi_ruby_write_binary", value)
    }

    fn direct_record(&mut self, _: RecordId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("record")]
    }

    fn encoded_record(&mut self, _: RecordId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("record")]
    }

    fn c_style_enum(&mut self, _: EnumId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("enum")]
    }

    fn data_enum(&mut self, _: EnumId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("enum")]
    }

    fn class_handle(&mut self, _: ClassId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("class handle")]
    }

    fn callback_handle(&mut self, _: CallbackId, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("callback handle")]
    }

    fn custom<F>(&mut self, _: CustomTypeId, _: &ValueRef, _: F) -> Vec<Self::Stmt>
    where
        F: FnOnce(&mut Self, &ValueRef) -> Vec<Self::Stmt>,
    {
        vec![unsupported("custom type")]
    }

    fn builtin(&mut self, _: BuiltinType, _: &ValueRef) -> Vec<Self::Stmt> {
        vec![unsupported("builtin type")]
    }

    fn optional(&mut self, _: &ValueRef, _: BinderId, _: Vec<Self::Stmt>) -> Vec<Self::Stmt> {
        vec![unsupported("optional value")]
    }

    fn sequence(
        &mut self,
        _: &ValueRef,
        _: &Op<ElementCount>,
        _: BinderId,
        _: Vec<Self::Stmt>,
    ) -> Vec<Self::Stmt> {
        vec![unsupported("sequence")]
    }

    fn tuple(&mut self, _: &ValueRef, _: Vec<Vec<Self::Stmt>>) -> Vec<Self::Stmt> {
        vec![unsupported("tuple")]
    }

    fn result(
        &mut self,
        _: &ValueRef,
        _: BinderId,
        _: Vec<Self::Stmt>,
        _: Vec<Self::Stmt>,
    ) -> Vec<Self::Stmt> {
        vec![unsupported("Result value")]
    }

    fn map(
        &mut self,
        _: MapKind,
        _: &ValueRef,
        _: BinderId,
        _: Vec<Self::Stmt>,
        _: BinderId,
        _: Vec<Self::Stmt>,
    ) -> Vec<Self::Stmt> {
        vec![unsupported("map")]
    }
}
