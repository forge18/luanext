pub mod lua51;
pub mod lua52;
pub mod lua53;
pub mod lua54;

use typedlua_parser::ast::expression::BinaryOp;
use typedlua_parser::string_interner::StringId;

/// Strategy for Lua version-specific code generation
pub trait CodeGenStrategy {
    /// Get the name of this strategy
    fn name(&self) -> &str;

    /// Generate a bitwise operation given the left and right expression strings
    fn generate_bitwise_op(&self, op: BinaryOp, left_expr: &str, right_expr: &str) -> String;

    /// Generate integer division given the left and right expression strings
    fn generate_integer_divide(&self, left_expr: &str, right_expr: &str) -> String;

    /// Generate continue statement (emulated if not supported)
    fn generate_continue(&self, label: Option<StringId>) -> String;

    /// Generate unary bitwise not given the operand expression string
    fn generate_unary_bitwise_not(&self, operand_expr: &str) -> String;

    /// Get optional preamble code (for library includes)
    fn emit_preamble(&self) -> Option<String>;

    /// Check if this strategy supports native bitwise operators
    fn supports_native_bitwise(&self) -> bool;

    /// Check if this strategy supports integer division
    fn supports_native_integer_divide(&self) -> bool;
}
