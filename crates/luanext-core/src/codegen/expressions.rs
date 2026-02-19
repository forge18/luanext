use super::dedent;
use super::CodeGenerator;
use crate::config::OptimizationLevel;
use luanext_parser::ast::expression::*;
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::prelude::{MatchArmBody, MatchExpression};

pub mod binary_ops;
pub mod calls;
pub mod literals;

/// Check if an expression is guaranteed to never be nil
/// Used for O2 null coalescing optimization to skip unnecessary nil checks
pub fn is_guaranteed_non_nil(expr: &Expression) -> bool {
    match &expr.kind {
        ExpressionKind::Literal(Literal::Boolean(_)) => true,
        ExpressionKind::Literal(Literal::Number(_)) => true,
        ExpressionKind::Literal(Literal::Integer(_)) => true,
        ExpressionKind::Literal(Literal::String(_)) => true,
        ExpressionKind::Object(_) => true,
        ExpressionKind::Array(_) => true,
        ExpressionKind::New(_, _, _) => true,
        ExpressionKind::Function(_) => true,
        ExpressionKind::Parenthesized(inner) => is_guaranteed_non_nil(inner),
        _ => false,
    }
}

/// Check if an expression is "simple" and can be safely evaluated twice
/// Simple expressions: identifiers, literals, and simple member/index access
pub fn is_simple_expression(expr: &Expression) -> bool {
    match &expr.kind {
        ExpressionKind::Identifier(_) => true,
        ExpressionKind::Literal(_) => true,
        ExpressionKind::Member(obj, _) => is_simple_expression(obj),
        ExpressionKind::Index(obj, index) => {
            is_simple_expression(obj) && is_simple_expression(index)
        }
        _ => false,
    }
}

/// Convert binary op to string
pub fn simple_binary_op_to_string(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Power => "^",
        BinaryOp::Concatenate => "..",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "~=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::NullCoalesce => unreachable!("null coalescing is handled separately"),
        BinaryOp::BitwiseAnd => "&",
        BinaryOp::BitwiseOr => "|",
        BinaryOp::BitwiseXor => "~",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::IntegerDivide => "//",
        BinaryOp::Instanceof => unreachable!("instanceof is handled separately"),
    }
}

/// Convert unary op to string
pub fn unary_op_to_string(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::Not => "not ",
        UnaryOp::Length => "#",
        UnaryOp::BitwiseNot => "~",
    }
}

impl CodeGenerator {
    pub fn is_guaranteed_non_nil(&self, expr: &Expression) -> bool {
        is_guaranteed_non_nil(expr)
    }

    pub fn is_simple_expression(&self, expr: &Expression) -> bool {
        is_simple_expression(expr)
    }

    pub fn simple_binary_op_to_string(&self, op: BinaryOp) -> &'static str {
        simple_binary_op_to_string(op)
    }

    pub fn unary_op_to_string(&self, op: UnaryOp) -> &'static str {
        unary_op_to_string(op)
    }

    pub fn expression_to_string(&mut self, expr: &Expression) -> String {
        let original_output = std::mem::take(self.emitter.output_mut());
        self.generate_expression(expr);
        std::mem::replace(self.emitter.output_mut(), original_output)
    }

    /// Generate expression to Lua code (main dispatcher)
    pub fn generate_expression(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Literal(lit) => self.generate_literal(lit),
            ExpressionKind::Identifier(name) => self.generate_identifier(*name),
            ExpressionKind::Binary(op, left, right) => {
                self.generate_binary_expression(*op, left, right);
            }
            ExpressionKind::Unary(op, operand) => {
                self.generate_unary_expression(*op, operand);
            }
            ExpressionKind::Assignment(target, op, value) => {
                self.generate_assignment_expression(target, *op, value);
            }
            ExpressionKind::Call(callee, args, type_args) => {
                // Check if this is an assertType intrinsic call
                if let ExpressionKind::Identifier(name_id) = &callee.kind {
                    let name = self.resolve(*name_id);
                    if name == "assertType" {
                        self.generate_assert_type_intrinsic(args, type_args);
                        return;
                    }
                }
                self.generate_call_expression(callee, args);
            }
            ExpressionKind::New(constructor, args, _type_args) => {
                self.write("(");
                self.generate_expression(constructor);
                self.write(".new(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_argument(arg);
                }
                self.write("))");
            }
            ExpressionKind::Member(object, member) => {
                if matches!(object.kind, ExpressionKind::SuperKeyword) {
                    if let Some(parent) = self.current_class_parent {
                        let parent_str = self.resolve(parent);
                        self.write(&parent_str);
                        self.write(".");
                        let member_str = self.resolve(member.node);
                        self.write(&member_str);
                    } else {
                        self.write("nil -- super used without parent class");
                    }
                } else {
                    self.generate_expression(object);
                    self.write(".");
                    let member_str = self.resolve(member.node);
                    self.write(&member_str);
                }
            }
            ExpressionKind::Index(object, index) => {
                self.generate_expression(object);
                self.write("[");
                self.generate_expression(index);
                self.write("]");
            }
            ExpressionKind::Array(elements) => {
                let has_spread = elements
                    .iter()
                    .any(|elem| matches!(elem, ArrayElement::Spread(_)));

                if !has_spread {
                    // Simple array literal without spread - use table constructor
                    self.write("{");
                    for (i, elem) in elements.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_array_element(elem);
                    }
                    self.write("}");
                } else {
                    // Array with spread elements - preallocate for known elements
                    let known_element_count = elements
                        .iter()
                        .filter(|elem| matches!(elem, ArrayElement::Expression(_)))
                        .count();

                    self.write("(function() ");

                    // Preallocate array with size hint for better performance
                    // For standard Lua, we create a table with known elements filled with nil
                    // This allocates the array part upfront, reducing reallocations
                    if known_element_count > 0 {
                        self.write("local __arr = {");
                        for i in 0..known_element_count {
                            if i > 0 {
                                self.write(", ");
                            }
                            self.write("nil");
                        }
                        self.write("} ");
                    } else {
                        self.write("local __arr = {} ");
                    }

                    for elem in elements.iter() {
                        match elem {
                            ArrayElement::Expression(expr) => {
                                self.write("table.insert(__arr, ");
                                self.generate_expression(expr);
                                self.write(") ");
                            }
                            ArrayElement::Spread(expr) => {
                                self.write("for _, __v in ipairs(");
                                self.generate_expression(expr);
                                self.write(") do table.insert(__arr, __v) end ");
                            }
                        }
                    }

                    self.write("return __arr end)()");
                }
            }
            ExpressionKind::Object(props) => {
                let has_spread = props
                    .iter()
                    .any(|prop| matches!(prop, ObjectProperty::Spread { .. }));

                if !has_spread {
                    self.write("{");
                    for (i, prop) in props.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_object_property(prop);
                    }
                    self.write("}");
                } else {
                    // Object with spread - use table preallocation optimization
                    // Collect known keys for size hint
                    let known_keys: Vec<String> = props
                        .iter()
                        .filter_map(|prop| match prop {
                            ObjectProperty::Property { key, .. } => {
                                Some(self.resolve(key.node).to_string())
                            }
                            ObjectProperty::Computed { .. } => None, // Dynamic keys can't be preallocated
                            ObjectProperty::Spread { .. } => None,
                        })
                        .collect();

                    self.write("(function() ");

                    // Preallocate object with known keys set to nil
                    // This hints the hash table size to the Lua VM, reducing rehashing overhead
                    if !known_keys.is_empty() {
                        self.write("local __obj = {");
                        for (i, key) in known_keys.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            self.write(key);
                            self.write(" = nil");
                        }
                        self.write("} ");
                    } else {
                        self.write("local __obj = {} ");
                    }

                    for prop in props.iter() {
                        match prop {
                            ObjectProperty::Property { key, value, .. } => {
                                self.write("__obj.");
                                let key_str = self.resolve(key.node);
                                self.write(&key_str);
                                self.write(" = ");
                                self.generate_expression(value);
                                self.write(" ");
                            }
                            ObjectProperty::Computed { key, value, .. } => {
                                self.write("__obj[");
                                self.generate_expression(key);
                                self.write("] = ");
                                self.generate_expression(value);
                                self.write(" ");
                            }
                            ObjectProperty::Spread { value, .. } => {
                                self.write("for __k, __v in pairs(");
                                self.generate_expression(value);
                                self.write(") do __obj[__k] = __v end ");
                            }
                        }
                    }

                    self.write("return __obj end)()");
                }
            }
            ExpressionKind::Function(func_expr) => {
                self.write("function(");
                for (i, param) in func_expr.parameters.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pattern(&param.pattern);
                }
                self.write(")\n");
                self.indent();
                self.generate_block(&func_expr.body);
                self.dedent();
                self.write_indent();
                self.write("end");
            }
            ExpressionKind::Arrow(arrow_expr) => {
                self.write("function(");
                for (i, param) in arrow_expr.parameters.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_pattern(&param.pattern);
                }
                self.write(")\n");
                self.indent();
                match &arrow_expr.body {
                    ArrowBody::Expression(expr) => {
                        self.write_indent();
                        self.write("return ");
                        self.generate_expression(expr);
                        self.writeln("");
                    }
                    ArrowBody::Block(block) => {
                        self.generate_block(block);
                    }
                }
                self.dedent();
                self.write_indent();
                self.write("end");
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                self.write("(");
                self.generate_expression(cond);
                self.write(" and ");
                self.generate_expression(then_expr);
                self.write(" or ");
                self.generate_expression(else_expr);
                self.write(")");
            }
            ExpressionKind::Match(match_expr) => {
                self.generate_match_expression(match_expr);
            }
            ExpressionKind::Pipe(left, right) => match &right.kind {
                ExpressionKind::Call(callee, arguments, _) => {
                    self.generate_expression(callee);
                    self.write("(");
                    self.generate_expression(left);
                    if !arguments.is_empty() {
                        self.write(", ");
                        for (i, arg) in arguments.iter().enumerate() {
                            if i > 0 {
                                self.write(", ");
                            }
                            if arg.is_spread {
                                self.write("table.unpack(");
                                self.generate_expression(&arg.value);
                                self.write(")");
                            } else {
                                self.generate_expression(&arg.value);
                            }
                        }
                    }
                    self.write(")");
                }
                _ => {
                    self.generate_expression(right);
                    self.write("(");
                    self.generate_expression(left);
                    self.write(")");
                }
            },
            ExpressionKind::MethodCall(object, method, args, _) => {
                self.generate_method_call_expression(object, method, args);
            }
            ExpressionKind::Parenthesized(expr) => {
                self.write("(");
                self.generate_expression(expr);
                self.write(")");
            }
            ExpressionKind::SelfKeyword => {
                self.write("self");
            }
            ExpressionKind::SuperKeyword => {
                if let Some(parent) = self.current_class_parent {
                    let parent_str = self.resolve(parent);
                    self.write(&parent_str);
                } else {
                    self.write("nil --[[super without parent class]]");
                }
            }
            ExpressionKind::Template(template_lit) => {
                self.write("(");
                let mut first = true;

                for part in template_lit.parts.iter() {
                    match part {
                        TemplatePart::String(s) => {
                            // Only dedent multi-line strings to preserve inline spacing
                            let processed = if s.contains('\n') {
                                dedent(s)
                            } else {
                                s.clone()
                            };
                            // Skip empty string parts (e.g. trailing empty from last interpolation)
                            if processed.is_empty() {
                                continue;
                            }
                            if !first {
                                self.write(" .. ");
                            }
                            first = false;
                            self.write("\"");
                            self.write(&processed.replace('\\', "\\\\").replace('"', "\\\""));
                            self.write("\"");
                        }
                        TemplatePart::Expression(expr) => {
                            if !first {
                                self.write(" .. ");
                            }
                            first = false;
                            self.write("tostring(");
                            self.generate_expression(expr);
                            self.write(")");
                        }
                    }
                }

                if first {
                    self.write("\"\"");
                }
                self.write(")");
            }
            ExpressionKind::TypeAssertion(expr, _type_annotation) => {
                self.generate_expression(expr);
            }
            ExpressionKind::OptionalMember(object, member) => {
                if self.optimization_level >= OptimizationLevel::Moderate
                    && self.is_guaranteed_non_nil(object)
                {
                    self.generate_expression(object);
                    self.write(".");
                    let member_str = self.resolve(member.node);
                    self.write(&member_str);
                } else if self.is_simple_expression(object) {
                    self.write("(");
                    self.generate_expression(object);
                    self.write(" and ");
                    self.generate_expression(object);
                    self.write(".");
                    let member_str = self.resolve(member.node);
                    self.write(&member_str);
                    self.write(" or nil)");
                } else {
                    self.write("(function() local __t = ");
                    self.generate_expression(object);
                    self.writeln("; if __t then return __t.");
                    self.write_indent();
                    let member_str = self.resolve(member.node);
                    self.write(&member_str);
                    self.writeln(" else return nil end end)()");
                }
            }
            ExpressionKind::OptionalIndex(object, index) => {
                if self.optimization_level >= OptimizationLevel::Moderate
                    && self.is_guaranteed_non_nil(object)
                {
                    self.generate_expression(object);
                    self.write("[");
                    self.generate_expression(index);
                    self.write("]");
                } else if self.is_simple_expression(object) {
                    self.write("(");
                    self.generate_expression(object);
                    self.write(" and ");
                    self.generate_expression(object);
                    self.write("[");
                    self.generate_expression(index);
                    self.write("] or nil)");
                } else {
                    self.write("(function() local __t = ");
                    self.generate_expression(object);
                    self.writeln("; if __t then return __t[");
                    self.write_indent();
                    self.generate_expression(index);
                    self.writeln("] else return nil end end)()");
                }
            }
            ExpressionKind::OptionalCall(callee, _args, _) => {
                if self.optimization_level >= OptimizationLevel::Moderate
                    && self.is_guaranteed_non_nil(callee)
                {
                    self.generate_expression(callee);
                    self.write("()");
                } else if self.is_simple_expression(callee) {
                    self.write("(");
                    self.generate_expression(callee);
                    self.write(" and ");
                    self.generate_expression(callee);
                    self.write("() or nil)");
                } else {
                    self.write("(function() local __t = ");
                    self.generate_expression(callee);
                    self.writeln("; if __t then return __t() else return nil end end)()");
                }
            }
            ExpressionKind::OptionalMethodCall(object, method, args, _) => {
                if self.optimization_level >= OptimizationLevel::Moderate
                    && self.is_guaranteed_non_nil(object)
                {
                    self.generate_expression(object);
                    self.write(":");
                    let method_str = self.resolve(method.node);
                    self.write(&method_str);
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_argument(arg);
                    }
                    self.write(")");
                } else if self.is_simple_expression(object) {
                    self.write("(");
                    self.generate_expression(object);
                    self.write(" and ");
                    self.generate_expression(object);
                    self.write(":");
                    let method_str = self.resolve(method.node);
                    self.write(&method_str);
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_argument(arg);
                    }
                    self.write(") or nil)");
                } else {
                    self.write("(function() local __t = ");
                    self.generate_expression(object);
                    self.write("; if __t then return __t:");
                    let method_str = self.resolve(method.node);
                    self.write(&method_str);
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_argument(arg);
                    }
                    self.writeln(") else return nil end end)()");
                }
            }
            ExpressionKind::Try(try_expr) => {
                self.write("(function() local __ok, __result = pcall(function() return ");
                self.generate_expression(try_expr.expression);
                self.writeln(" end); ");
                self.write("if __ok then return __result else ");
                let var_name = self.resolve(try_expr.catch_variable.node);
                self.write("local ");
                self.write(&var_name);
                self.write(" = __result; return ");
                self.generate_expression(try_expr.catch_expression);
                self.writeln(" end end)()");
            }
            ExpressionKind::ErrorChain(left, right) => {
                self.write("(function() local __ok, __result = pcall(function() return ");
                self.generate_expression(left);
                self.writeln(" end); ");
                self.write("if __ok then return __result else return ");
                self.generate_expression(right);
                self.writeln(" end end)()");
            }
        }
    }

    pub fn generate_null_coalesce(&mut self, left: &Expression, right: &Expression) {
        if self.optimization_level >= OptimizationLevel::Moderate
            && self.is_guaranteed_non_nil(left)
        {
            self.generate_expression(left);
            return;
        }

        if self.is_simple_expression(left) {
            self.write("(");
            self.generate_expression(left);
            self.write(" ~= nil and ");
            self.generate_expression(left);
            self.write(" or ");
            self.generate_expression(right);
            self.write(")");
        } else {
            self.write("(function() local __left = ");
            self.generate_expression(left);
            self.writeln(";");
            self.write_indent();
            self.write("return __left ~= nil and __left or ");
            self.generate_expression(right);
            self.writeln("");
            self.write_indent();
            self.write("end)()");
        }
    }

    pub fn generate_match_expression(&mut self, match_expr: &MatchExpression) {
        self.write("(function()");
        self.writeln("");
        self.indent();

        self.write_indent();
        self.write("local __match_value = ");
        self.generate_expression(match_expr.value);
        self.writeln("");

        // Pre-bind identifier patterns that have guards, so guard expressions
        // can reference the bound variable (e.g., `n if n > 100`)
        let mut pre_bound_guard_idents = std::collections::HashSet::new();
        for arm in match_expr.arms.iter() {
            if arm.guard.is_some() {
                if let Pattern::Identifier(ident) = &arm.pattern {
                    let ident_str = self.resolve(ident.node);
                    if pre_bound_guard_idents.insert(ident_str.clone()) {
                        self.write_indent();
                        self.write("local ");
                        self.write(&ident_str);
                        self.write(" = __match_value");
                        self.writeln("");
                    }
                }
            }
        }

        for (i, arm) in match_expr.arms.iter().enumerate() {
            self.write_indent();
            if i == 0 {
                self.write("if ");
            } else {
                self.write("elseif ");
            }

            self.generate_pattern_match(&arm.pattern, "__match_value");

            if let Some(guard) = &arm.guard {
                self.write(" and (");
                self.generate_expression(guard);
                self.write(")");
            }

            self.write(" then");
            self.writeln("");
            self.indent();

            // Skip re-binding identifier patterns that were pre-bound for guards
            let skip_binding =
                arm.guard.is_some() && matches!(&arm.pattern, Pattern::Identifier(_));
            if !skip_binding {
                self.generate_pattern_bindings(&arm.pattern, "__match_value");
            }

            self.write_indent();
            match &arm.body {
                MatchArmBody::Expression(expr) => {
                    self.write("return ");
                    self.generate_expression(expr);
                    self.writeln("");
                }
                MatchArmBody::Block(block) => {
                    for stmt in block.statements.iter() {
                        self.generate_statement(stmt);
                    }
                    self.write_indent();
                    self.writeln("return nil");
                }
            }

            self.dedent();
        }

        self.write_indent();
        self.writeln("else");
        self.indent();
        self.write_indent();
        self.writeln("error(\"Non-exhaustive match\")");
        self.dedent();
        self.write_indent();
        self.writeln("end");

        self.dedent();
        self.write_indent();
        self.write("end)()");
    }

    pub fn generate_pattern_match(&mut self, pattern: &Pattern, value_var: &str) {
        match pattern {
            Pattern::Wildcard(_) => {
                self.write("true");
            }
            Pattern::Identifier(_) => {
                self.write("true");
            }
            Pattern::Literal(lit, _) => {
                self.write(value_var);
                self.write(" == ");
                self.generate_literal(lit);
            }
            Pattern::Array(array_pattern) => {
                self.write("type(");
                self.write(value_var);
                self.write(") == \"table\"");

                for (i, elem) in array_pattern.elements.iter().enumerate() {
                    match elem {
                        luanext_parser::ast::pattern::ArrayPatternElement::Pattern(
                            luanext_parser::ast::pattern::PatternWithDefault {
                                pattern: pat, ..
                            },
                        ) => {
                            self.write(" and ");
                            let index_expr = format!("{}[{}]", value_var, i + 1);
                            self.generate_pattern_match(pat, &index_expr);
                        }
                        luanext_parser::ast::pattern::ArrayPatternElement::Rest(_) => {}
                        luanext_parser::ast::pattern::ArrayPatternElement::Hole => {}
                    }
                }
            }
            Pattern::Object(_) => {
                self.write("type(");
                self.write(value_var);
                self.write(") == \"table\"");
            }
            Pattern::Or(or_pattern) => {
                self.write("(");
                for (i, alt) in or_pattern.alternatives.iter().enumerate() {
                    if i > 0 {
                        self.write(" or ");
                    }
                    self.generate_pattern_match(alt, value_var);
                }
                self.write(")");
            }
            Pattern::Template(template_pattern) => {
                use luanext_parser::ast::pattern::TemplatePatternPart;

                // Count captures
                let capture_count = template_pattern
                    .parts
                    .iter()
                    .filter(|p| matches!(p, TemplatePatternPart::Capture(_)))
                    .count();

                if capture_count == 0 {
                    // No captures - should have been converted to literal by parser
                    unreachable!(
                        "Template pattern with no captures should be converted to literal"
                    );
                }

                // Generate Lua pattern string
                let lua_pattern = self.generate_lua_pattern(template_pattern);

                // Generate capture variable declarations
                let capture_vars = (1..=capture_count)
                    .map(|i| format!("__capture_{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");

                // Generate: __capture_1, __capture_2 = string.match(__match_value, "pattern")
                self.write(&capture_vars);
                self.write(" = string.match(");
                self.write(value_var);
                self.write(", \"");
                self.write(&lua_pattern);
                self.write("\")");

                // Check if match succeeded
                self.write(" and __capture_1 ~= nil");
            }
        }
    }

    pub fn generate_pattern_bindings(&mut self, pattern: &Pattern, value_var: &str) {
        match pattern {
            Pattern::Identifier(ident) => {
                self.write_indent();
                self.write("local ");
                let ident_str = self.resolve(ident.node);
                self.write(&ident_str);
                self.write(" = ");
                self.write(value_var);
                self.writeln("");
            }
            Pattern::Array(array_pattern) => {
                for (i, elem) in array_pattern.elements.iter().enumerate() {
                    match elem {
                        luanext_parser::ast::pattern::ArrayPatternElement::Pattern(
                            luanext_parser::ast::pattern::PatternWithDefault {
                                pattern: pat, ..
                            },
                        ) => {
                            let index_expr = format!("{}[{}]", value_var, i + 1);
                            self.generate_pattern_bindings(pat, &index_expr);
                        }
                        luanext_parser::ast::pattern::ArrayPatternElement::Rest(ident) => {
                            self.write_indent();
                            self.write("local ");
                            let ident_str = self.resolve(ident.node);
                            self.write(&ident_str);
                            self.write(" = {table.unpack(");
                            self.write(value_var);
                            self.write(", ");
                            self.write(&(i + 1).to_string());
                            self.write(")}");
                            self.writeln("");
                        }
                        luanext_parser::ast::pattern::ArrayPatternElement::Hole => {}
                    }
                }
            }
            Pattern::Object(object_pattern) => {
                for prop in object_pattern.properties.iter() {
                    if let Some(value_pattern) = &prop.value {
                        let key_str = self.resolve(prop.key.node);
                        let prop_expr = format!("{}.{}", value_var, key_str);
                        self.generate_pattern_bindings(value_pattern, &prop_expr);
                    } else {
                        self.write_indent();
                        self.write("local ");
                        let key_str = self.resolve(prop.key.node);
                        self.write(&key_str);
                        self.write(" = ");
                        self.write(value_var);
                        self.write(".");
                        self.write(&key_str);
                        self.writeln("");
                    }
                }
            }
            Pattern::Wildcard(_) | Pattern::Literal(_, _) => {}
            Pattern::Or(or_pattern) => {
                if let Some(first) = or_pattern.alternatives.first() {
                    self.generate_pattern_bindings(first, value_var);
                }
            }
            Pattern::Template(template_pattern) => {
                use luanext_parser::ast::pattern::TemplatePatternPart;

                let mut capture_idx = 1;
                for part in template_pattern.parts.iter() {
                    if let TemplatePatternPart::Capture(ident) = part {
                        self.write_indent();
                        self.write("local ");
                        let ident_str = self.resolve(ident.node);
                        self.write(&ident_str);
                        self.write(&format!(" = __capture_{}", capture_idx));
                        self.writeln("");
                        capture_idx += 1;
                    }
                }
            }
        }
    }

    fn generate_lua_pattern(
        &self,
        template: &luanext_parser::ast::pattern::TemplatePattern,
    ) -> String {
        use luanext_parser::ast::pattern::TemplatePatternPart;

        let mut pattern = String::from("^");
        let parts = template.parts;

        for (i, part) in parts.iter().enumerate() {
            match part {
                TemplatePatternPart::String(s) => {
                    pattern.push_str(&self.escape_lua_pattern(s));
                }
                TemplatePatternPart::Capture(_) => {
                    // Find next literal to determine capture class
                    let next_delimiter_char = parts.iter().skip(i + 1).find_map(|p| match p {
                        TemplatePatternPart::String(s) if !s.is_empty() => s.chars().next(),
                        _ => None,
                    });

                    if let Some(delimiter) = next_delimiter_char {
                        // Non-greedy: stop at delimiter
                        pattern.push_str(&format!(
                            "([^{}]+)",
                            self.escape_lua_pattern(&delimiter.to_string())
                        ));
                    } else {
                        // Greedy: consume rest
                        pattern.push_str("(.+)");
                    }
                }
            }
        }

        pattern.push('$');
        pattern
    }

    fn escape_lua_pattern(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 10);
        for ch in s.chars() {
            match ch {
                // Lua pattern magic characters need % prefix
                '^' | '$' | '(' | ')' | '%' | '.' | '[' | ']' | '*' | '+' | '-' | '?' => {
                    result.push('%');
                    result.push(ch);
                }
                _ => result.push(ch),
            }
        }
        result
    }

    /// Generate code for assertType<T>(value) intrinsic
    ///
    /// Emits runtime type checking code that validates the value matches type T,
    /// throwing an error with a descriptive message if the check fails.
    ///
    /// Phase 2: Implements primitive types (string, number, boolean, nil, table, integer)
    /// Phase 3: Will add unions, optionals, classes, interfaces, literals
    pub fn generate_assert_type_intrinsic(
        &mut self,
        args: &[luanext_parser::prelude::Argument],
        type_args: &Option<&[luanext_parser::ast::types::Type]>,
    ) {
        // Validation should have been done in type checker
        // Here we just generate the runtime check

        if args.is_empty() || type_args.is_none() {
            // Fallback: just emit the value if type checking failed
            if !args.is_empty() {
                self.generate_expression(&args[0].value);
            } else {
                self.write("nil");
            }
            return;
        }

        let type_arg = &type_args.unwrap()[0];
        let value_expr = &args[0].value;

        // Generate the runtime check wrapped in an IIFE that returns the value on success
        self.write("(function() local __val = ");
        self.generate_expression(value_expr);
        self.write("; ");

        // Generate type check based on the type argument
        use luanext_parser::ast::types::TypeKind;
        match &type_arg.kind {
            TypeKind::Primitive(prim) => {
                self.generate_primitive_type_check(prim);
            }
            TypeKind::Union(members) => {
                // Phase 3: Union types
                self.generate_union_type_check(members);
            }
            TypeKind::Nullable(inner) => {
                // Phase 3: Optional types (T?)
                self.generate_nullable_type_check(inner);
            }
            TypeKind::Literal(lit) => {
                // Phase 3: Literal types
                self.generate_literal_type_check(lit);
            }
            TypeKind::Reference(type_ref) => {
                // Phase 3: Class/interface types
                self.generate_reference_type_check(type_ref);
            }
            _ => {
                // Unknown type - skip check and just return value
                self.write("return __val end)()");
                return;
            }
        }

        self.write(" return __val end)()");
    }

    /// Generate runtime check for primitive types
    fn generate_primitive_type_check(&mut self, prim: &luanext_parser::ast::types::PrimitiveType) {
        use luanext_parser::ast::types::PrimitiveType;
        let (expected_type, lua_type_check) = match prim {
            PrimitiveType::String => ("string", "type(__val) ~= \"string\""),
            PrimitiveType::Number => ("number", "type(__val) ~= \"number\""),
            PrimitiveType::Boolean => ("boolean", "type(__val) ~= \"boolean\""),
            PrimitiveType::Nil => ("nil", "__val ~= nil"),
            PrimitiveType::Table => ("table", "type(__val) ~= \"table\""),
            PrimitiveType::Integer => {
                // Lua 5.3+: math.type(x) == "integer"
                // Lua 5.1/5.2: type(x) == "number" and x % 1 == 0
                // We'll emit both checks with a fallback
                (
                    "integer",
                    "(type(__val) ~= \"number\" or (math.type and math.type(__val) ~= \"integer\" or not math.type and __val % 1 ~= 0))",
                )
            }
            PrimitiveType::Unknown
            | PrimitiveType::Void
            | PrimitiveType::Never
            | PrimitiveType::Coroutine
            | PrimitiveType::Thread => {
                // Skip check for unknown/void/never/coroutine/thread
                return;
            }
        };

        self.write("if ");
        self.write(lua_type_check);
        self.write(" then error(\"Type assertion failed: expected ");
        self.write(expected_type);
        self.write(", got \" .. type(__val)) end;");
    }

    /// Generate runtime check for union types
    fn generate_union_type_check(&mut self, members: &[luanext_parser::ast::types::Type]) {
        use luanext_parser::ast::types::{PrimitiveType, TypeKind};

        // Build a list of type checks that will be OR'd together
        // For union types like `string | number`, we need: type(__val) == "string" or type(__val) == "number"
        let mut type_checks = Vec::new();
        let mut type_names: Vec<String> = Vec::new();

        for member in members {
            match &member.kind {
                TypeKind::Primitive(prim) => {
                    let (type_name, check) = match prim {
                        PrimitiveType::String => ("string", "type(__val) == \"string\""),
                        PrimitiveType::Number => ("number", "type(__val) == \"number\""),
                        PrimitiveType::Boolean => ("boolean", "type(__val) == \"boolean\""),
                        PrimitiveType::Table => ("table", "type(__val) == \"table\""),
                        PrimitiveType::Nil => ("nil", "__val == nil"),
                        PrimitiveType::Integer => {
                            type_names.push("integer".to_string());
                            type_checks.push("(type(__val) == \"number\" and (math.type and math.type(__val) == \"integer\" or not math.type and __val % 1 == 0))".to_string());
                            continue;
                        }
                        _ => continue, // Skip unknown/void/never/etc.
                    };
                    type_names.push(type_name.to_string());
                    type_checks.push(check.to_string());
                }
                TypeKind::Literal(lit) => {
                    // Handle literal types in unions
                    use luanext_parser::ast::expression::Literal;
                    match lit {
                        Literal::Nil => {
                            type_names.push("nil".to_string());
                            type_checks.push("__val == nil".to_string());
                        }
                        Literal::Boolean(b) => {
                            let bool_str = if *b { "true" } else { "false" };
                            type_names.push(bool_str.to_string());
                            type_checks.push(format!("__val == {}", bool_str));
                        }
                        Literal::String(s) => {
                            type_names.push(format!("\"{}\"", s));
                            type_checks.push(format!("__val == \"{}\"", s.replace('\"', "\\\"")));
                        }
                        Literal::Number(n) => {
                            let num_str = n.to_string();
                            type_names.push(num_str.clone());
                            type_checks.push(format!("__val == {}", num_str));
                        }
                        Literal::Integer(i) => {
                            let int_str = i.to_string();
                            type_names.push(int_str.clone());
                            type_checks.push(format!("__val == {}", int_str));
                        }
                    }
                }
                TypeKind::Reference(type_ref) => {
                    let ref_name = self.resolve(type_ref.name.node);
                    if self.interface_members.contains_key(&ref_name) {
                        // Interface in union: check table type (structural check too complex for union OR)
                        type_names.push(ref_name);
                        type_checks.push("type(__val) == \"table\"".to_string());
                    } else {
                        // Class in union: check metatable
                        type_checks.push(format!(
                            "(type(__val) == \"table\" and getmetatable(__val) == {})",
                            ref_name
                        ));
                        type_names.push(ref_name);
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if type_checks.is_empty() {
            // No checkable types in union - skip check
            return;
        }

        // Generate: if not (check1 or check2 or ...) then error(...) end
        self.write("if not (");
        for (i, check) in type_checks.iter().enumerate() {
            if i > 0 {
                self.write(" or ");
            }
            self.write(check);
        }
        self.write(") then error(\"Type assertion failed: expected ");
        for (i, name) in type_names.iter().enumerate() {
            if i > 0 {
                self.write(" | ");
            }
            self.write(name);
        }
        self.write(", got \" .. type(__val)) end;");
    }

    /// Generate runtime check for nullable types (T?)
    fn generate_nullable_type_check(&mut self, inner: &luanext_parser::ast::types::Type) {
        use luanext_parser::ast::types::{PrimitiveType, TypeKind};

        // For nullable types, we need: __val == nil or (inner_check passes)
        // Example: string? → __val == nil or type(__val) == "string"

        match &inner.kind {
            TypeKind::Primitive(prim) => {
                let (type_name, check) = match prim {
                    PrimitiveType::String => ("string", "type(__val) == \"string\""),
                    PrimitiveType::Number => ("number", "type(__val) == \"number\""),
                    PrimitiveType::Boolean => ("boolean", "type(__val) == \"boolean\""),
                    PrimitiveType::Table => ("table", "type(__val) == \"table\""),
                    PrimitiveType::Integer => {
                        self.write("if __val ~= nil and not (type(__val) == \"number\" and (math.type and math.type(__val) == \"integer\" or not math.type and __val % 1 == 0)) then error(\"Type assertion failed: expected integer?, got \" .. type(__val)) end;");
                        return;
                    }
                    _ => {
                        // For Unknown/Void/Never, skip check
                        return;
                    }
                };

                self.write("if __val ~= nil and ");
                self.write(check);
                self.write(" == false then error(\"Type assertion failed: expected ");
                self.write(type_name);
                self.write("?, got \" .. type(__val)) end;");
            }
            TypeKind::Literal(lit) => {
                // For literal types that are nullable
                use luanext_parser::ast::expression::Literal;
                match lit {
                    Literal::Nil => {
                        // nil? is just nil, so check is always true
                    }
                    Literal::Boolean(b) => {
                        let bool_str = if *b { "true" } else { "false" };
                        self.write("if __val ~= nil and __val ~= ");
                        self.write(bool_str);
                        self.write(" then error(\"Type assertion failed: expected ");
                        self.write(bool_str);
                        self.write("?, got \" .. tostring(__val)) end;");
                    }
                    Literal::String(s) => {
                        self.write("if __val ~= nil and __val ~= \"");
                        self.write(&s.replace('\"', "\\\""));
                        self.write("\" then error(\"Type assertion failed: expected \\\"");
                        self.write(&s.replace('\"', "\\\""));
                        self.write("\\\"?, got \" .. tostring(__val)) end;");
                    }
                    Literal::Number(n) => {
                        let num_str = n.to_string();
                        self.write("if __val ~= nil and __val ~= ");
                        self.write(&num_str);
                        self.write(" then error(\"Type assertion failed: expected ");
                        self.write(&num_str);
                        self.write("?, got \" .. tostring(__val)) end;");
                    }
                    Literal::Integer(i) => {
                        let int_str = i.to_string();
                        self.write("if __val ~= nil and __val ~= ");
                        self.write(&int_str);
                        self.write(" then error(\"Type assertion failed: expected ");
                        self.write(&int_str);
                        self.write("?, got \" .. tostring(__val)) end;");
                    }
                }
            }
            TypeKind::Reference(type_ref) => {
                let ref_name = self.resolve(type_ref.name.node);
                if let Some(members) = self.interface_members.get(&ref_name).cloned() {
                    // Nullable interface: nil or structural check
                    self.write("if __val ~= nil then if type(__val) ~= \"table\" then error(\"Type assertion failed: expected ");
                    self.write(&ref_name);
                    self.write("?, got \" .. type(__val))");
                    for member in &members {
                        self.write(" elseif __val.");
                        self.write(member);
                        self.write(" == nil then error(\"Type assertion failed: ");
                        self.write(&ref_name);
                        self.write(" requires property '");
                        self.write(member);
                        self.write("'\")");
                    }
                    self.write(" end end;");
                } else {
                    // Nullable class: nil or metatable check
                    self.write(
                        "if __val ~= nil and (type(__val) ~= \"table\" or getmetatable(__val) ~= ",
                    );
                    self.write(&ref_name);
                    self.write(") then error(\"Type assertion failed: expected ");
                    self.write(&ref_name);
                    self.write("?, got \" .. type(__val)) end;");
                }
            }
            _ => {
                // For other complex types, skip the check
            }
        }
    }

    /// Generate runtime check for literal types
    fn generate_literal_type_check(&mut self, lit: &luanext_parser::ast::expression::Literal) {
        use luanext_parser::ast::expression::Literal;
        match lit {
            Literal::Nil => {
                // For nil literal type, check if value is NOT nil
                self.write("if __val ~= nil then error(\"Type assertion failed: expected nil, got \" .. type(__val)) end;");
            }
            Literal::Boolean(b) => {
                // For boolean literal type, check exact value
                let bool_str = if *b { "true" } else { "false" };
                self.write("if __val ~= ");
                self.write(bool_str);
                self.write(" then error(\"Type assertion failed: expected ");
                self.write(bool_str);
                self.write(", got \" .. tostring(__val)) end;");
            }
            Literal::String(_s) => {
                // For string literal type, check exact value
                self.write("if __val ~= ");
                self.generate_literal(lit);
                self.write(" then error(\"Type assertion failed: expected ");
                // Escape the string for error message
                self.generate_literal(lit);
                self.write(", got \" .. tostring(__val)) end;");
            }
            Literal::Number(n) => {
                // For number literal type, check exact value
                let num_str = n.to_string();
                self.write("if __val ~= ");
                self.write(&num_str);
                self.write(" then error(\"Type assertion failed: expected ");
                self.write(&num_str);
                self.write(", got \" .. tostring(__val)) end;");
            }
            Literal::Integer(i) => {
                // For integer literal type, check exact value
                let int_str = i.to_string();
                self.write("if __val ~= ");
                self.write(&int_str);
                self.write(" then error(\"Type assertion failed: expected ");
                self.write(&int_str);
                self.write(", got \" .. tostring(__val)) end;");
            }
        }
    }

    /// Generate runtime check for reference types (classes/interfaces)
    fn generate_reference_type_check(
        &mut self,
        type_ref: &luanext_parser::ast::types::TypeReference,
    ) {
        let type_name = self.resolve(type_ref.name.node);

        if let Some(members) = self.interface_members.get(&type_name).cloned() {
            // Interface: structural check — table + required properties
            self.write("if type(__val) ~= \"table\" then error(\"Type assertion failed: expected ");
            self.write(&type_name);
            self.write(", got \" .. type(__val))");
            for member in &members {
                self.write(" elseif __val.");
                self.write(member);
                self.write(" == nil then error(\"Type assertion failed: ");
                self.write(&type_name);
                self.write(" requires property '");
                self.write(member);
                self.write("'\")");
            }
            self.write(" end;");
        } else {
            // Class: table + metatable check
            self.write("if type(__val) ~= \"table\" then error(\"Type assertion failed: expected ");
            self.write(&type_name);
            self.write(", got \" .. type(__val)) elseif getmetatable(__val) ~= ");
            self.write(&type_name);
            self.write(" then error(\"Type assertion failed: expected instance of ");
            self.write(&type_name);
            self.write("\") end;");
        }
    }
}
