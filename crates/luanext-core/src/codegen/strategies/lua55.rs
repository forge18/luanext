use super::CodeGenStrategy;
use luanext_parser::ast::expression::BinaryOp;
use luanext_parser::string_interner::StringId;

/// Code generation strategy for Lua 5.5
/// - Native bitwise operators (& | ~ << >>)
/// - Supports goto/labels
/// - Native integer division
/// - Native continue statement (no goto hack needed)
/// - Native global declaration keyword
pub struct Lua55Strategy;

impl CodeGenStrategy for Lua55Strategy {
    fn name(&self) -> &str {
        "Lua 5.5"
    }

    fn generate_bitwise_op(&self, op: BinaryOp, left_expr: &str, right_expr: &str) -> String {
        let op_str = match op {
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "~",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
            _ => unreachable!("Not a bitwise operator"),
        };

        format!("({} {} {})", left_expr, op_str, right_expr)
    }

    fn generate_integer_divide(&self, left_expr: &str, right_expr: &str) -> String {
        format!("({} // {})", left_expr, right_expr)
    }

    fn generate_continue(&self, _label: Option<StringId>) -> String {
        "continue".to_string()
    }

    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String {
        format!("~{}", operand_expr)
    }

    fn emit_preamble(&self) -> Option<String> {
        None
    }

    fn supports_native_bitwise(&self) -> bool {
        true
    }

    fn supports_native_integer_divide(&self) -> bool {
        true
    }

    fn supports_goto(&self) -> bool {
        true
    }

    fn supports_native_continue(&self) -> bool {
        true
    }

    fn global_declaration_prefix(&self) -> Option<&str> {
        Some("global ")
    }
}
