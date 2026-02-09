use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::{Expression, ExpressionKind};
use luanext_parser::ast::statement::{Block, EnumDeclaration, Statement};
use luanext_parser::string_interner::StringId;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct RichEnumOptimizationPass {
    /// Maps enum name to its fields
    enum_fields: FxHashMap<StringId, FxHashSet<StringId>>,
    /// Maps enum name to simple methods that can be inlined
    enum_simple_methods: FxHashMap<StringId, FxHashSet<StringId>>,
}

impl RichEnumOptimizationPass {
    pub fn new() -> Self {
        Self {
            enum_fields: FxHashMap::default(),
            enum_simple_methods: FxHashMap::default(),
        }
    }

    /// Collect information about rich enums in the program
    fn analyze_enums<'arena>(&mut self, program: &MutableProgram<'arena>) {
        self.enum_fields.clear();
        self.enum_simple_methods.clear();

        for stmt in &program.statements {
            if let Statement::Enum(enum_decl) = stmt {
                if self.is_rich_enum(enum_decl) {
                    let enum_name = enum_decl.name.node;

                    // Collect fields
                    let mut fields = FxHashSet::default();
                    for field in enum_decl.fields.iter() {
                        fields.insert(field.name.node);
                    }
                    self.enum_fields.insert(enum_name, fields);

                    // Collect simple methods
                    let mut simple_methods = FxHashSet::default();
                    for method in enum_decl.methods.iter() {
                        if Self::is_simple_enum_method(method) {
                            simple_methods.insert(method.name.node);
                        }
                    }
                    if !simple_methods.is_empty() {
                        self.enum_simple_methods.insert(enum_name, simple_methods);
                    }
                }
            }
        }
    }

    /// Check if an enum method is simple (single return statement, no params)
    fn is_simple_enum_method<'arena>(
        method: &luanext_parser::ast::statement::EnumMethod<'arena>,
    ) -> bool {
        // Must have no parameters (or only self)
        if !method.parameters.is_empty() {
            return false;
        }

        // Check if body is a simple block with just a return statement
        let statements = &method.body.statements;

        if statements.len() != 1 {
            return false;
        }

        matches!(&statements[0], Statement::Return(_))
    }
}

impl<'arena> WholeProgramPass<'arena> for RichEnumOptimizationPass {
    fn name(&self) -> &'static str {
        "rich-enum-optimization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Moderate
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_ENUMS
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        // First, analyze enums in the program
        self.analyze_enums(program);

        let rich_enum_count = self.enum_fields.len();

        if rich_enum_count > 0 {
            tracing::debug!(
                "Found {} rich enum(s) with {} total simple methods",
                rich_enum_count,
                self.enum_simple_methods
                    .values()
                    .map(|s| s.len())
                    .sum::<usize>()
            );
        }

        // Apply optimizations to statements
        let mut changed = false;
        for stmt in &mut program.statements {
            changed |= self.optimize_statement(stmt, arena);
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl RichEnumOptimizationPass {
    fn is_rich_enum<'arena>(&self, enum_decl: &EnumDeclaration<'arena>) -> bool {
        !enum_decl.fields.is_empty()
            || enum_decl.constructor.is_some()
            || !enum_decl.methods.is_empty()
    }

    fn optimize_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Function(func) => {
                let mut stmts: Vec<_> = func.body.statements.to_vec();
                let mut changed = false;
                for s in &mut stmts {
                    changed |= self.optimize_statement(s, arena);
                }
                if changed {
                    func.body.statements = arena.alloc_slice_clone(&stmts);
                }
                changed
            }
            Statement::If(if_stmt) => {
                let mut changed = self.optimize_expression(&mut if_stmt.condition, arena);
                changed |= self.optimize_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.optimize_expression(&mut else_if.condition, arena);
                    eic |= self.optimize_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.optimize_block(else_block, arena);
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = self.optimize_expression(&mut while_stmt.condition, arena);
                changed |= self.optimize_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => {
                use luanext_parser::ast::statement::ForStatement;
                match &**for_stmt {
                    ForStatement::Numeric(for_num_ref) => {
                        let mut new_num = (**for_num_ref).clone();
                        let changed = self.optimize_block(&mut new_num.body, arena);
                        if changed {
                            *stmt = Statement::For(
                                arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                            );
                        }
                        changed
                    }
                    ForStatement::Generic(for_gen_ref) => {
                        let mut new_gen = for_gen_ref.clone();
                        let changed = self.optimize_block(&mut new_gen.body, arena);
                        if changed {
                            *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        }
                        changed
                    }
                }
            }
            Statement::Repeat(repeat_stmt) => {
                let mut changed = self.optimize_expression(&mut repeat_stmt.until, arena);
                changed |= self.optimize_block(&mut repeat_stmt.body, arena);
                changed
            }
            Statement::Return(return_stmt) => {
                let mut vals: Vec<_> = return_stmt.values.to_vec();
                let mut changed = false;
                for value in &mut vals {
                    changed |= self.optimize_expression(value, arena);
                }
                if changed {
                    return_stmt.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            Statement::Expression(expr) => self.optimize_expression(expr, arena),
            Statement::Block(block) => self.optimize_block(block, arena),
            _ => false,
        }
    }

    fn optimize_block<'arena>(&mut self, block: &mut Block<'arena>, arena: &'arena Bump) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.optimize_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn optimize_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::Member(obj, _field) => {
                // Optimize field access on enum members
                // This is already optimal at codegen level (direct table access)
                // But we traverse for nested expressions
                let mut new_obj = (**obj).clone();
                let changed = self.optimize_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), _field.clone());
                }
                changed
            }
            ExpressionKind::MethodCall(obj, method_name, args, type_args) => {
                // Traverse object and arguments
                let mut new_obj = (**obj).clone();
                let mut changed = self.optimize_expression(&mut new_obj, arena);

                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.optimize_expression(&mut arg.value, arena);
                }

                // TODO: Could inline simple enum methods here in future
                // For now, just update traversed children
                if changed || args_changed {
                    expr.kind = ExpressionKind::MethodCall(
                        arena.alloc(new_obj),
                        method_name.clone(),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }

                changed
            }
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                let mut changed = self.optimize_expression(&mut new_func, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.optimize_expression(&mut arg.value, arena);
                }
                if changed || args_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_func),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.optimize_expression(&mut new_left, arena);
                let right_changed = self.optimize_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                let changed = self.optimize_expression(&mut new_operand, arena);
                if changed {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                }
                changed
            }
            ExpressionKind::Assignment(left, op, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.optimize_expression(&mut new_left, arena);
                let right_changed = self.optimize_expression(&mut new_right, arena);
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
                let c1 = self.optimize_expression(&mut new_cond, arena);
                let c2 = self.optimize_expression(&mut new_then, arena);
                let c3 = self.optimize_expression(&mut new_else, arena);
                if c1 || c2 || c3 {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                }
                c1 || c2 || c3
            }
            ExpressionKind::Index(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let c1 = self.optimize_expression(&mut new_obj, arena);
                let c2 = self.optimize_expression(&mut new_index, arena);
                if c1 || c2 {
                    expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                }
                c1 || c2
            }
            // For other expression types, we don't optimize (return false)
            _ => false,
        }
    }
}

impl Default for RichEnumOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use luanext_parser::ast::pattern::Pattern;
    use luanext_parser::ast::statement::{
        Block, EnumConstructor, EnumDeclaration, EnumField, EnumMember, Parameter,
    };
    use luanext_parser::ast::types::{PrimitiveType, Type, TypeKind};
    use luanext_parser::ast::Spanned;
    use luanext_parser::span::Span;
    use luanext_parser::string_interner::StringInterner;

    fn number_type() -> Type<'static> {
        Type::new(TypeKind::Primitive(PrimitiveType::Number), Span::dummy())
    }

    fn create_test_program_with_rich_enum<'arena>(arena: &'arena Bump) -> MutableProgram<'arena> {
        let interner = StringInterner::new();

        let mercury_name = Spanned::new(interner.get_or_intern("Mercury"), Span::dummy());
        let mass_field = Spanned::new(interner.get_or_intern("mass"), Span::dummy());
        let radius_field = Spanned::new(interner.get_or_intern("radius"), Span::dummy());

        let member_args = arena.alloc_slice_clone(&[
            Expression::new(
                ExpressionKind::Literal(Literal::Number(3.303e23)),
                Span::dummy(),
            ),
            Expression::new(
                ExpressionKind::Literal(Literal::Number(2.4397e6)),
                Span::dummy(),
            ),
        ]);

        let members = arena.alloc_slice_clone(&[EnumMember {
            name: mercury_name.clone(),
            arguments: member_args,
            value: None,
            span: Span::dummy(),
        }]);

        let fields = arena.alloc_slice_clone(&[
            EnumField {
                name: mass_field,
                type_annotation: number_type(),
                span: Span::dummy(),
            },
            EnumField {
                name: radius_field,
                type_annotation: number_type(),
                span: Span::dummy(),
            },
        ]);

        let params = arena.alloc_slice_clone(&[
            Parameter {
                pattern: Pattern::Identifier(Spanned::new(
                    interner.get_or_intern("mass"),
                    Span::dummy(),
                )),
                type_annotation: Some(number_type()),
                default: None,
                is_rest: false,
                is_optional: false,
                span: Span::dummy(),
            },
            Parameter {
                pattern: Pattern::Identifier(Spanned::new(
                    interner.get_or_intern("radius"),
                    Span::dummy(),
                )),
                type_annotation: Some(number_type()),
                default: None,
                is_rest: false,
                is_optional: false,
                span: Span::dummy(),
            },
        ]);

        let empty_stmts: &[Statement<'arena>] = arena.alloc_slice_clone(&[]);

        let enum_decl = EnumDeclaration {
            name: Spanned::new(interner.get_or_intern("Planet"), Span::dummy()),
            members,
            fields,
            constructor: Some(EnumConstructor {
                parameters: params,
                body: Block {
                    statements: empty_stmts,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }),
            methods: arena.alloc_slice_clone(&[]),
            implements: arena.alloc_slice_clone(&[]),
            span: Span::dummy(),
        };

        MutableProgram {
            statements: vec![Statement::Enum(enum_decl)],
            span: Span::dummy(),
        }
    }

    fn create_test_program_with_simple_enum<'arena>(arena: &'arena Bump) -> MutableProgram<'arena> {
        let interner = StringInterner::new();

        let members = arena.alloc_slice_clone(&[EnumMember {
            name: Spanned::new(interner.get_or_intern("Red"), Span::dummy()),
            arguments: arena.alloc_slice_clone(&[]),
            value: Some(luanext_parser::ast::statement::EnumValue::Number(1.0)),
            span: Span::dummy(),
        }]);

        let enum_decl = EnumDeclaration {
            name: Spanned::new(interner.get_or_intern("Color"), Span::dummy()),
            members,
            fields: arena.alloc_slice_clone(&[]),
            constructor: None,
            methods: arena.alloc_slice_clone(&[]),
            implements: arena.alloc_slice_clone(&[]),
            span: Span::dummy(),
        };

        MutableProgram {
            statements: vec![Statement::Enum(enum_decl)],
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_rich_enum_detection() {
        let arena = Bump::new();
        let mut pass = RichEnumOptimizationPass::new();
        let mut program = create_test_program_with_rich_enum(&arena);
        let result = pass.run(&mut program, &arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_enum_not_rich() {
        let arena = Bump::new();
        let mut pass = RichEnumOptimizationPass::new();
        let mut program = create_test_program_with_simple_enum(&arena);
        let result = pass.run(&mut program, &arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pass_analyzes_enums() {
        let arena = Bump::new();
        let mut pass = RichEnumOptimizationPass::new();
        let mut program = create_test_program_with_rich_enum(&arena);
        let result = pass.run(&mut program, &arena);
        assert!(result.is_ok());

        // Verify the pass analyzed the enum
        assert_eq!(pass.enum_fields.len(), 1, "Should have found 1 rich enum");
    }

    #[test]
    fn test_field_detection() {
        let arena = Bump::new();
        let mut pass = RichEnumOptimizationPass::new();
        let mut program = create_test_program_with_rich_enum(&arena);

        pass.run(&mut program, &arena).unwrap();

        // Verify that the pass tracked exactly 1 enum with 2 fields
        assert_eq!(pass.enum_fields.len(), 1, "Should have found 1 rich enum");

        // Get the first (and only) enum's fields
        let fields = pass.enum_fields.values().next().unwrap();
        assert_eq!(fields.len(), 2, "Should have 2 fields (mass and radius)");
    }
}
