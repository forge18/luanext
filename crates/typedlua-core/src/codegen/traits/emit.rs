use crate::codegen::emitter::Emitter;

/// Trait for emitting Lua code from AST nodes.
/// This allows composition of code generation logic and makes it easy to test.
pub trait Emit {
    fn emit(&self, emitter: &mut Emitter);
}

pub trait EmitStatement {
    fn emit_statement(&self, emitter: &mut Emitter);
}

pub trait EmitExpression {
    fn emit_expression(&self, emitter: &mut Emitter);
}

pub trait EmitPattern {
    fn emit_pattern(&self, emitter: &mut Emitter);
}

pub trait EmitType {
    fn emit_type(&self, emitter: &mut Emitter);
}

/// Helper trait for nodes that can optionally emit (emit or skip)
pub trait MaybeEmit {
    fn maybe_emit(&self, emitter: &mut Emitter);
}

impl<T: Emit> MaybeEmit for Option<T> {
    fn maybe_emit(&self, emitter: &mut Emitter) {
        if let Some(ref t) = *self {
            t.emit(emitter);
        }
    }
}

impl<T: Emit> MaybeEmit for &T {
    fn maybe_emit(&self, emitter: &mut Emitter) {
        (*self).emit(emitter);
    }
}

impl<T: Emit> MaybeEmit for &Option<T> {
    fn maybe_emit(&self, emitter: &mut Emitter) {
        if let Some(ref t) = *self {
            t.emit(emitter);
        }
    }
}

/// Emit a slice of items
pub fn emit_list<T: Emit>(items: &[T], emitter: &mut Emitter, separator: &str) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            emitter.write(separator);
        }
        item.emit(emitter);
    }
}

/// Emit a slice of items with newlines between them
pub fn emit_list_lines<T: Emit>(items: &[T], emitter: &mut Emitter) {
    for item in items {
        item.emit(emitter);
        emitter.writeln("");
    }
}

/// Emit a comma-separated list
pub fn emit_comma_list<T: Emit>(items: &[T], emitter: &mut Emitter) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            emitter.write(", ");
        }
        item.emit(emitter);
    }
}

/// Emit an indented block
pub fn emit_block<F>(content: F, emitter: &mut Emitter)
where
    F: FnOnce(&mut Emitter),
{
    emitter.writeln("");
    emitter.indent();
    content(emitter);
    emitter.dedent();
}
