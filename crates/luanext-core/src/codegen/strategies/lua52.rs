use super::CodeGenStrategy;
use luanext_parser::ast::expression::BinaryOp;
use luanext_parser::string_interner::StringId;

/// Code generation strategy for Lua 5.2
/// - Bitwise operators via bit32 library
/// - Supports goto/labels
/// - No integer division
pub struct Lua52Strategy;

impl CodeGenStrategy for Lua52Strategy {
    fn name(&self) -> &str {
        "Lua 5.2"
    }

    fn generate_bitwise_op(&self, op: BinaryOp, left_expr: &str, right_expr: &str) -> String {
        let func = match op {
            BinaryOp::BitwiseAnd => "band",
            BinaryOp::BitwiseOr => "bor",
            BinaryOp::BitwiseXor => "bxor",
            BinaryOp::ShiftLeft => "lshift",
            BinaryOp::ShiftRight => "rshift",
            _ => unreachable!("Not a bitwise operator"),
        };

        format!("bit32.{}({}, {})", func, left_expr, right_expr)
    }

    fn generate_integer_divide(&self, left_expr: &str, right_expr: &str) -> String {
        format!("math.floor({} / {})", left_expr, right_expr)
    }

    fn generate_continue(&self, _label: Option<StringId>) -> String {
        "goto __continue".to_string()
    }

    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String {
        format!("bit32.bnot({})", operand_expr)
    }

    fn emit_preamble(&self) -> Option<String> {
        // Emit bit32 polyfill so Lua 5.2 output is self-contained.
        // On a real Lua 5.2 runtime, this harmlessly shadows the built-in bit32.
        Some(luanext_runtime::bitwise::for_lua52().to_string())
    }

    fn supports_native_bitwise(&self) -> bool {
        false
    }

    fn supports_native_integer_divide(&self) -> bool {
        false
    }

    fn supports_goto(&self) -> bool {
        true
    }
}
