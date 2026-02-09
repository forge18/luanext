use crate::config::OptimizationLevel;
use crate::MutableProgram;
use bumpalo::Bump;

use crate::optimizer::{StmtVisitor, WholeProgramPass};
use luanext_parser::ast::expression::{Expression, ExpressionKind, ReceiverClassInfo};
use luanext_parser::ast::statement::Statement;
use luanext_parser::span::Span;
use luanext_parser::string_interner::StringInterner;
use std::sync::Arc;

pub struct MethodToFunctionConversionPass {
    interner: Arc<StringInterner>,
}

impl MethodToFunctionConversionPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self { interner }
    }

    fn convert_in_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Function(func) => {
                let mut stmts: Vec<_> = func.body.statements.to_vec();
                let mut changed = false;
                for s in &mut stmts {
                    changed |= self.convert_in_statement(s, arena);
                }
                if changed {
                    func.body.statements = arena.alloc_slice_clone(&stmts);
                }
                changed
            }
            Statement::If(if_stmt) => {
                let mut changed = self.convert_in_expression(&mut if_stmt.condition, arena);
                changed |= self.convert_in_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.convert_in_expression(&mut else_if.condition, arena);
                    eic |= self.convert_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.convert_in_block(else_block, arena);
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = self.convert_in_expression(&mut while_stmt.condition, arena);
                changed |= self.convert_in_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => {
                use luanext_parser::ast::statement::ForStatement;
                match &**for_stmt {
                    ForStatement::Numeric(for_num_ref) => {
                        let mut new_num = (**for_num_ref).clone();
                        let changed = self.convert_in_block(&mut new_num.body, arena);
                        if changed {
                            *stmt = Statement::For(
                                arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                            );
                        }
                        changed
                    }
                    ForStatement::Generic(for_gen_ref) => {
                        let mut new_gen = for_gen_ref.clone();
                        let changed = self.convert_in_block(&mut new_gen.body, arena);
                        if changed {
                            *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        }
                        changed
                    }
                }
            }
            Statement::Repeat(repeat_stmt) => {
                let mut changed = self.convert_in_expression(&mut repeat_stmt.until, arena);
                changed |= self.convert_in_block(&mut repeat_stmt.body, arena);
                changed
            }
            Statement::Return(return_stmt) => {
                let mut vals: Vec<_> = return_stmt.values.to_vec();
                let mut changed = false;
                for value in &mut vals {
                    changed |= self.convert_in_expression(value, arena);
                }
                if changed {
                    return_stmt.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            Statement::Expression(expr) => self.convert_in_expression(expr, arena),
            Statement::Block(block) => self.convert_in_block(block, arena),
            Statement::Try(try_stmt) => {
                let mut changed = self.convert_in_block(&mut try_stmt.try_block, arena);
                let mut new_clauses: Vec<_> = try_stmt.catch_clauses.to_vec();
                let mut clauses_changed = false;
                for clause in &mut new_clauses {
                    clauses_changed |= self.convert_in_block(&mut clause.body, arena);
                }
                if clauses_changed {
                    try_stmt.catch_clauses = arena.alloc_slice_clone(&new_clauses);
                    changed = true;
                }
                if let Some(finally) = &mut try_stmt.finally_block {
                    changed |= self.convert_in_block(finally, arena);
                }
                changed
            }
            _ => false,
        }
    }

    fn convert_in_block<'arena>(
        &mut self,
        block: &mut luanext_parser::ast::statement::Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.convert_in_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn convert_in_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                let mut changed = self.convert_in_expression(&mut new_func, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.convert_in_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;
                if changed || args_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_func),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::MethodCall(obj, method_name, args, type_args) => {
                let method_name = method_name.clone();
                let mut new_obj = (**obj).clone();
                let mut changed = self.convert_in_expression(&mut new_obj, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.convert_in_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;

                if changed || args_changed {
                    expr.kind = ExpressionKind::MethodCall(
                        arena.alloc(new_obj.clone()),
                        method_name.clone(),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }

                if let Some(receiver_info) = &expr.receiver_class {
                    if let Some(converted) = self.convert_method_call_to_function_call(
                        &new_obj,
                        receiver_info,
                        &method_name,
                        &new_args,
                        expr.span,
                        arena,
                    ) {
                        expr.kind = converted;
                        expr.receiver_class = None;
                        changed = true;
                    }
                }

                changed
            }
            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.convert_in_expression(&mut new_left, arena);
                let right_changed = self.convert_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                let changed = self.convert_in_expression(&mut new_operand, arena);
                if changed {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                }
                changed
            }
            ExpressionKind::Assignment(left, op, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.convert_in_expression(&mut new_left, arena);
                let right_changed = self.convert_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind = ExpressionKind::Assignment(
                        arena.alloc(new_left),
                        op,
                        arena.alloc(new_right),
                    );
                }
                left_changed || right_changed
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                let mut new_cond = (**cond).clone();
                let mut new_then = (**then_expr).clone();
                let mut new_else = (**else_expr).clone();
                let c1 = self.convert_in_expression(&mut new_cond, arena);
                let c2 = self.convert_in_expression(&mut new_then, arena);
                let c3 = self.convert_in_expression(&mut new_else, arena);
                if c1 || c2 || c3 {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                }
                c1 || c2 || c3
            }
            ExpressionKind::Pipe(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.convert_in_expression(&mut new_left, arena);
                let right_changed = self.convert_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Match(match_expr) => {
                let mut new_value = (*match_expr.value).clone();
                let mut changed = self.convert_in_expression(&mut new_value, arena);
                let mut new_arms: Vec<_> = match_expr.arms.to_vec();
                let mut arms_changed = false;
                for arm in &mut new_arms {
                    match &mut arm.body {
                        luanext_parser::ast::expression::MatchArmBody::Expression(arm_expr) => {
                            let mut new_arm_expr = (**arm_expr).clone();
                            if self.convert_in_expression(&mut new_arm_expr, arena) {
                                arm.body =
                                    luanext_parser::ast::expression::MatchArmBody::Expression(
                                        arena.alloc(new_arm_expr),
                                    );
                                arms_changed = true;
                            }
                        }
                        luanext_parser::ast::expression::MatchArmBody::Block(block) => {
                            arms_changed |= self.convert_in_block(block, arena);
                        }
                    }
                }
                if changed || arms_changed {
                    expr.kind =
                        ExpressionKind::Match(luanext_parser::ast::expression::MatchExpression {
                            value: arena.alloc(new_value),
                            arms: arena.alloc_slice_clone(&new_arms),
                            span: match_expr.span,
                        });
                    changed = true;
                }
                changed
            }
            ExpressionKind::Arrow(arrow) => {
                let mut new_arrow = arrow.clone();
                let mut changed = false;
                let mut new_params: Vec<_> = new_arrow.parameters.to_vec();
                let mut params_changed = false;
                for param in &mut new_params {
                    if let Some(default) = &mut param.default {
                        params_changed |= self.convert_in_expression(default, arena);
                    }
                }
                if params_changed {
                    new_arrow.parameters = arena.alloc_slice_clone(&new_params);
                    changed = true;
                }
                match &mut new_arrow.body {
                    luanext_parser::ast::expression::ArrowBody::Expression(body_expr) => {
                        let mut new_body = (**body_expr).clone();
                        if self.convert_in_expression(&mut new_body, arena) {
                            new_arrow.body = luanext_parser::ast::expression::ArrowBody::Expression(
                                arena.alloc(new_body),
                            );
                            changed = true;
                        }
                    }
                    luanext_parser::ast::expression::ArrowBody::Block(block) => {
                        changed |= self.convert_in_block(block, arena);
                    }
                }
                if changed {
                    expr.kind = ExpressionKind::Arrow(new_arrow);
                }
                changed
            }
            ExpressionKind::New(callee, args, type_args) => {
                let mut new_callee = (**callee).clone();
                let mut changed = self.convert_in_expression(&mut new_callee, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.convert_in_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;
                if changed || args_changed {
                    expr.kind = ExpressionKind::New(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Try(try_expr) => {
                let mut new_expression = (*try_expr.expression).clone();
                let mut new_catch = (*try_expr.catch_expression).clone();
                let c1 = self.convert_in_expression(&mut new_expression, arena);
                let c2 = self.convert_in_expression(&mut new_catch, arena);
                if c1 || c2 {
                    expr.kind =
                        ExpressionKind::Try(luanext_parser::ast::expression::TryExpression {
                            expression: arena.alloc(new_expression),
                            catch_variable: try_expr.catch_variable.clone(),
                            catch_expression: arena.alloc(new_catch),
                            span: try_expr.span,
                        });
                }
                c1 || c2
            }
            ExpressionKind::ErrorChain(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.convert_in_expression(&mut new_left, arena);
                let right_changed = self.convert_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::ErrorChain(arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::OptionalMember(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                let changed = self.convert_in_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::OptionalMember(arena.alloc(new_obj), member);
                }
                changed
            }
            ExpressionKind::OptionalIndex(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let c1 = self.convert_in_expression(&mut new_obj, arena);
                let c2 = self.convert_in_expression(&mut new_index, arena);
                if c1 || c2 {
                    expr.kind =
                        ExpressionKind::OptionalIndex(arena.alloc(new_obj), arena.alloc(new_index));
                }
                c1 || c2
            }
            ExpressionKind::OptionalCall(obj, args, type_args) => {
                let mut new_obj = (**obj).clone();
                let mut changed = self.convert_in_expression(&mut new_obj, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.convert_in_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;
                if changed || args_changed {
                    expr.kind = ExpressionKind::OptionalCall(
                        arena.alloc(new_obj),
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::OptionalMethodCall(obj, method_name, args, type_args) => {
                let method_name = method_name.clone();
                let mut new_obj = (**obj).clone();
                let mut changed = self.convert_in_expression(&mut new_obj, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.convert_in_expression(&mut arg.value, arena);
                }
                let type_args = *type_args;
                if changed || args_changed {
                    expr.kind = ExpressionKind::OptionalMethodCall(
                        arena.alloc(new_obj),
                        method_name,
                        arena.alloc_slice_clone(&new_args),
                        type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Member(..)
            | ExpressionKind::Index(..)
            | ExpressionKind::Identifier(..)
            | ExpressionKind::Literal(..)
            | ExpressionKind::SelfKeyword
            | ExpressionKind::SuperKeyword
            | ExpressionKind::Template(..)
            | ExpressionKind::TypeAssertion(..)
            | ExpressionKind::Array(..)
            | ExpressionKind::Object(..)
            | ExpressionKind::Function(..)
            | ExpressionKind::Parenthesized(..) => false,
        }
    }

    fn convert_method_call_to_function_call<'arena>(
        &self,
        obj: &Expression<'arena>,
        receiver_info: &ReceiverClassInfo,
        method_name: &luanext_parser::ast::Ident,
        args: &[luanext_parser::ast::expression::Argument<'arena>],
        span: Span,
        arena: &'arena Bump,
    ) -> Option<ExpressionKind<'arena>> {
        let class_name_str = self.interner.resolve(receiver_info.class_name);
        let class_id = self.interner.get_or_intern(&class_name_str);

        let class_expr = Expression {
            kind: ExpressionKind::Member(
                arena.alloc(Expression {
                    kind: ExpressionKind::Identifier(class_id),
                    span,
                    annotated_type: None,
                    receiver_class: None,
                }),
                method_name.clone(),
            ),
            span,
            annotated_type: None,
            receiver_class: None,
        };

        let new_args: Vec<_> = std::iter::once(luanext_parser::ast::expression::Argument {
            value: obj.clone(),
            is_spread: false,
            span,
        })
        .chain(args.iter().cloned())
        .collect();

        Some(ExpressionKind::Call(
            arena.alloc(class_expr),
            arena.alloc_slice_clone(&new_args),
            None,
        ))
    }
}

impl<'arena> StmtVisitor<'arena> for MethodToFunctionConversionPass {
    fn visit_stmt(&mut self, stmt: &mut Statement<'arena>, arena: &'arena Bump) -> bool {
        self.convert_in_statement(stmt, arena)
    }
}

impl<'arena> WholeProgramPass<'arena> for MethodToFunctionConversionPass {
    fn name(&self) -> &'static str {
        "method-to-function-conversion"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Moderate
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;
        for stmt in &mut program.statements {
            changed |= self.convert_in_statement(stmt, arena);
        }
        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for MethodToFunctionConversionPass {
    fn default() -> Self {
        Self {
            interner: Arc::new(StringInterner::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bumpalo::Bump;
    use luanext_parser::ast::expression::{ExpressionKind, Literal};
    use luanext_parser::ast::statement::{Block, Statement};
    use luanext_parser::ast::types::{PrimitiveType, Type, TypeKind};
    use luanext_parser::ast::Spanned;
    use luanext_parser::span::Span;

    #[test]
    fn test_method_call_to_function_call_conversion() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = MethodToFunctionConversionPass::new(interner.clone());

        let obj_id = interner.get_or_intern("myObj");
        let method_id = interner.get_or_intern("calculate");
        let class_id = interner.get_or_intern("Calculator");

        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let arg_expr = Expression {
            kind: ExpressionKind::Literal(Literal::Number(42.0)),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let arguments = arena.alloc_slice_clone(&[luanext_parser::ast::expression::Argument {
            value: arg_expr,
            is_spread: false,
            span: Span::dummy(),
        }]);

        let receiver_class = Some(ReceiverClassInfo {
            class_name: class_id,
            is_static: false,
        });

        let expr = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arguments,
                None,
            ),
            span: Span::dummy(),
            annotated_type: Some(Type::new(
                TypeKind::Primitive(PrimitiveType::Number),
                Span::dummy(),
            )),
            receiver_class,
        };

        let stmts = arena.alloc_slice_clone(&[Statement::Expression(expr)]);
        let mut block = Block {
            statements: stmts,
            span: Span::dummy(),
        };

        let result = pass.convert_in_block(&mut block, &arena);
        assert!(result, "Should have made changes");

        if let Statement::Expression(converted_expr) = &block.statements[0] {
            if let ExpressionKind::Call(callee, args, _) = &converted_expr.kind {
                if let ExpressionKind::Member(class_expr, method) = &callee.kind {
                    let class_str = interner.resolve(match &class_expr.kind {
                        ExpressionKind::Identifier(id) => *id,
                        _ => panic!("Expected identifier"),
                    });
                    assert_eq!(class_str, "Calculator", "Class name should be Calculator");

                    let method_str = interner.resolve(method.node);
                    assert_eq!(method_str, "calculate", "Method name should be calculate");

                    assert_eq!(args.len(), 2, "Should have 2 args (obj + original arg)");
                    assert!(
                        matches!(args[0].value.kind, ExpressionKind::Identifier(_)),
                        "First arg should be the object"
                    );
                } else {
                    panic!("Expected Member expression");
                }
            } else {
                panic!("Expected Call expression");
            }
        }
    }

    #[test]
    fn test_preserves_receiver_class_info() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = MethodToFunctionConversionPass::new(interner.clone());

        let obj_id = interner.get_or_intern("myObj");
        let method_id = interner.get_or_intern("test");
        let class_id = interner.get_or_intern("TestClass");

        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let receiver_class = Some(ReceiverClassInfo {
            class_name: class_id,
            is_static: false,
        });

        let empty_args: &[luanext_parser::ast::expression::Argument] = arena.alloc_slice_clone(&[]);

        let expr = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                empty_args,
                None,
            ),
            span: Span::dummy(),
            annotated_type: Some(Type::new(
                TypeKind::Primitive(PrimitiveType::Number),
                Span::dummy(),
            )),
            receiver_class,
        };

        let stmts = arena.alloc_slice_clone(&[Statement::Expression(expr)]);
        let mut block = Block {
            statements: stmts,
            span: Span::dummy(),
        };

        pass.convert_in_block(&mut block, &arena);

        if let Statement::Expression(converted_expr) = &block.statements[0] {
            assert!(
                converted_expr.receiver_class.is_none(),
                "receiver_class should be cleared after conversion"
            );
        }
    }
}
