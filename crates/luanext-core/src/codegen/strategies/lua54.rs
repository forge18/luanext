use super::CodeGenStrategy;
use typedlua_parser::ast::expression::BinaryOp;
use typedlua_parser::string_interner::StringId;

/// Code generation strategy for Lua 5.4
/// - Native bitwise operators (& | ~ << >>)
/// - Supports goto/labels
/// - Native integer division
/// - Const expressions (generated as-is)
pub struct Lua54Strategy;

impl CodeGenStrategy for Lua54Strategy {
    fn name(&self) -> &str {
        "Lua 5.4"
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
        "goto __continue".to_string()
    }

    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String {
        format!("~{}", operand_expr)
    }

    fn emit_preamble(&self) -> Option<String> {
        None // All features are native
    }

    fn supports_native_bitwise(&self) -> bool {
        true
    }

    fn supports_native_integer_divide(&self) -> bool {
        true
    }
}
