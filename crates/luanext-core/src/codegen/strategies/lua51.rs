use super::CodeGenStrategy;
use typedlua_parser::ast::expression::BinaryOp;
use typedlua_parser::string_interner::StringId;

/// Code generation strategy for Lua 5.1
/// - No native bitwise operators (requires helpers)
/// - No goto/continue
/// - No integer division
pub struct Lua51Strategy;

impl CodeGenStrategy for Lua51Strategy {
    fn name(&self) -> &str {
        "Lua 5.1"
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

        format!("_bit_{}({}, {})", func, left_expr, right_expr)
    }

    fn generate_integer_divide(&self, left_expr: &str, right_expr: &str) -> String {
        format!("math.floor({} / {})", left_expr, right_expr)
    }

    fn generate_continue(&self, _label: Option<StringId>) -> String {
        "goto __continue".to_string()
    }

    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String {
        format!("_bit_bnot({})", operand_expr)
    }

    fn emit_preamble(&self) -> Option<String> {
        Some(typedlua_runtime::bitwise::for_lua51().to_string())
    }

    fn supports_native_bitwise(&self) -> bool {
        false
    }

    fn supports_native_integer_divide(&self) -> bool {
        false
    }
}
