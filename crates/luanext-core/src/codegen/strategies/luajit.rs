use super::CodeGenStrategy;
use luanext_parser::ast::expression::BinaryOp;
use luanext_parser::string_interner::StringId;

/// Code generation strategy for LuaJIT
/// - Bitwise operators via built-in `bit` library (NOT pure-Lua helpers)
/// - Supports goto/labels (LuaJIT extension, unlike standard Lua 5.1)
/// - No native integer division (uses math.floor)
/// - Based on Lua 5.1 with extensions
pub struct LuaJITStrategy;

impl CodeGenStrategy for LuaJITStrategy {
    fn name(&self) -> &str {
        "LuaJIT"
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

        format!("bit.{}({}, {})", func, left_expr, right_expr)
    }

    fn generate_integer_divide(&self, left_expr: &str, right_expr: &str) -> String {
        format!("math.floor({} / {})", left_expr, right_expr)
    }

    fn generate_continue(&self, _label: Option<StringId>) -> String {
        "goto __continue".to_string()
    }

    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String {
        format!("bit.bnot({})", operand_expr)
    }

    fn emit_preamble(&self) -> Option<String> {
        None // `bit` library is built into LuaJIT, no preamble needed
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
