use super::super::CodeGenerator;
use luanext_parser::ast::expression::{AssignmentOp, BinaryOp, ExpressionKind};

impl CodeGenerator {
    pub fn generate_binary_expression(
        &mut self,
        op: BinaryOp,
        left: &luanext_parser::ast::expression::Expression,
        right: &luanext_parser::ast::expression::Expression,
    ) {
        match op {
            BinaryOp::NullCoalesce => {
                self.generate_null_coalesce(left, right);
            }

            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::Power
            | BinaryOp::Concatenate
            | BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LessThan
            | BinaryOp::LessThanOrEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanOrEqual
            | BinaryOp::And
            | BinaryOp::Or => {
                self.write("(");
                self.generate_expression(left);
                self.write(" ");
                self.write(self.simple_binary_op_to_string(op));
                self.write(" ");
                self.generate_expression(right);
                self.write(")");
            }

            BinaryOp::Instanceof => {
                self.write("(type(");
                self.generate_expression(left);
                self.write(") == \"table\" and getmetatable(");
                self.generate_expression(left);
                self.write(") == ");
                self.generate_expression(right);
                self.write(")");
            }

            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => {
                let left_str = self.expression_to_string(left);
                let right_str = self.expression_to_string(right);
                let result = self.strategy.generate_bitwise_op(op, &left_str, &right_str);
                self.write(&result);
            }

            BinaryOp::IntegerDivide => {
                let left_str = self.expression_to_string(left);
                let right_str = self.expression_to_string(right);
                let result = self.strategy.generate_integer_divide(&left_str, &right_str);
                self.write(&result);
            }
        }
    }

    pub fn generate_unary_expression(
        &mut self,
        op: luanext_parser::ast::expression::UnaryOp,
        operand: &luanext_parser::ast::expression::Expression,
    ) {
        if op == luanext_parser::ast::expression::UnaryOp::BitwiseNot
            && !self.strategy.supports_native_bitwise()
        {
            let operand_str = self.expression_to_string(operand);
            let result = self.strategy.generate_unary_bitwise_not(&operand_str);
            self.write(&result);
        } else {
            let op_str = self.unary_op_to_string(op).to_string();
            self.write(&op_str);
            self.generate_expression(operand);
        }
    }

    pub fn generate_assignment_expression(
        &mut self,
        target: &luanext_parser::ast::expression::Expression,
        op: AssignmentOp,
        value: &luanext_parser::ast::expression::Expression,
    ) {
        // Handle optional chaining as assignment target:
        // obj?.x = value  →  if obj ~= nil then obj.x = value end
        // obj?.[k] = value  →  if obj ~= nil then obj[k] = value end
        match &target.kind {
            ExpressionKind::OptionalMember(object, member) => {
                self.generate_optional_member_assignment(object, member, op, value);
                return;
            }
            ExpressionKind::OptionalIndex(object, index) => {
                self.generate_optional_index_assignment(object, index, op, value);
                return;
            }
            _ => {}
        }

        match op {
            AssignmentOp::Assign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(value);
            }
            AssignmentOp::AddAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" + ");
                self.generate_expression(value);
            }
            AssignmentOp::SubtractAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" - ");
                self.generate_expression(value);
            }
            AssignmentOp::MultiplyAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" * ");
                self.generate_expression(value);
            }
            AssignmentOp::DivideAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" / ");
                self.generate_expression(value);
            }
            AssignmentOp::ModuloAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" % ");
                self.generate_expression(value);
            }
            AssignmentOp::PowerAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" ^ ");
                self.generate_expression(value);
            }
            AssignmentOp::ConcatenateAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" .. ");
                self.generate_expression(value);
            }
            AssignmentOp::BitwiseAndAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" & ");
                self.generate_expression(value);
            }
            AssignmentOp::BitwiseOrAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" | ");
                self.generate_expression(value);
            }
            AssignmentOp::FloorDivideAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" // ");
                self.generate_expression(value);
            }
            AssignmentOp::LeftShiftAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" << ");
                self.generate_expression(value);
            }
            AssignmentOp::RightShiftAssign => {
                self.generate_expression(target);
                self.write(" = ");
                self.generate_expression(target);
                self.write(" >> ");
                self.generate_expression(value);
            }
        }
    }

    /// Generate assignment to an optional member: obj?.x = value
    /// Emits: if obj ~= nil then obj.x = value end
    /// For complex expressions, uses a temp var to avoid double evaluation.
    fn generate_optional_member_assignment(
        &mut self,
        object: &luanext_parser::ast::expression::Expression,
        member: &luanext_parser::ast::Ident,
        op: AssignmentOp,
        value: &luanext_parser::ast::expression::Expression,
    ) {
        let member_str = self.resolve(member.node);
        if self.is_simple_expression(object) {
            // Simple case: obj?.x = value → if obj ~= nil then obj.x = value end
            self.write("if ");
            self.generate_expression(object);
            self.write(" ~= nil then ");
            self.generate_expression(object);
            self.write(".");
            self.write(&member_str);
            self.write_assignment_op_rhs(op, object, Some(&member_str), None, value);
            self.write(" end");
        } else {
            // Complex case: use temp var to avoid double evaluation
            self.write("(function() local __t = ");
            self.generate_expression(object);
            self.write("; if __t ~= nil then __t.");
            self.write(&member_str);
            self.write_assignment_op_rhs_temp(op, "__t", Some(&member_str), None, value);
            self.write(" end end)()");
        }
    }

    /// Generate assignment to an optional index: obj?.[k] = value
    /// Emits: if obj ~= nil then obj[k] = value end
    fn generate_optional_index_assignment(
        &mut self,
        object: &luanext_parser::ast::expression::Expression,
        index: &luanext_parser::ast::expression::Expression,
        op: AssignmentOp,
        value: &luanext_parser::ast::expression::Expression,
    ) {
        if self.is_simple_expression(object) {
            self.write("if ");
            self.generate_expression(object);
            self.write(" ~= nil then ");
            self.generate_expression(object);
            self.write("[");
            self.generate_expression(index);
            self.write("]");
            self.write_assignment_op_rhs(op, object, None, Some(index), value);
            self.write(" end");
        } else {
            self.write("(function() local __t = ");
            self.generate_expression(object);
            self.write("; if __t ~= nil then __t[");
            self.generate_expression(index);
            self.write("]");
            self.write_assignment_op_rhs_temp(op, "__t", None, Some(index), value);
            self.write(" end end)()");
        }
    }

    /// Write the RHS of an assignment with the correct operator.
    /// For simple =, just writes " = value".
    /// For compound (+=, etc.), writes " = target.member OP value".
    fn write_assignment_op_rhs(
        &mut self,
        op: AssignmentOp,
        object: &luanext_parser::ast::expression::Expression,
        member: Option<&str>,
        index: Option<&luanext_parser::ast::expression::Expression>,
        value: &luanext_parser::ast::expression::Expression,
    ) {
        self.write(" = ");
        if op != AssignmentOp::Assign {
            // Emit target reference for compound assignment
            self.generate_expression(object);
            if let Some(m) = member {
                self.write(".");
                self.write(m);
            } else if let Some(idx) = index {
                self.write("[");
                self.generate_expression(idx);
                self.write("]");
            }
            self.write(&format!(" {} ", Self::compound_op_str(op)));
        }
        self.generate_expression(value);
    }

    /// Write the RHS using a temp variable name for compound assignment.
    fn write_assignment_op_rhs_temp(
        &mut self,
        op: AssignmentOp,
        temp_var: &str,
        member: Option<&str>,
        index: Option<&luanext_parser::ast::expression::Expression>,
        value: &luanext_parser::ast::expression::Expression,
    ) {
        self.write(" = ");
        if op != AssignmentOp::Assign {
            self.write(temp_var);
            if let Some(m) = member {
                self.write(".");
                self.write(m);
            } else if let Some(idx) = index {
                self.write("[");
                self.generate_expression(idx);
                self.write("]");
            }
            self.write(&format!(" {} ", Self::compound_op_str(op)));
        }
        self.generate_expression(value);
    }

    /// Get the Lua operator string for a compound assignment op.
    fn compound_op_str(op: AssignmentOp) -> &'static str {
        match op {
            AssignmentOp::Assign => "",
            AssignmentOp::AddAssign => "+",
            AssignmentOp::SubtractAssign => "-",
            AssignmentOp::MultiplyAssign => "*",
            AssignmentOp::DivideAssign => "/",
            AssignmentOp::ModuloAssign => "%",
            AssignmentOp::PowerAssign => "^",
            AssignmentOp::ConcatenateAssign => "..",
            AssignmentOp::BitwiseAndAssign => "&",
            AssignmentOp::BitwiseOrAssign => "|",
            AssignmentOp::FloorDivideAssign => "//",
            AssignmentOp::LeftShiftAssign => "<<",
            AssignmentOp::RightShiftAssign => ">>",
        }
    }
}
