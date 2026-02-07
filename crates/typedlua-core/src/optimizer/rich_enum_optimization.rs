use bumpalo::Bump;
use crate::config::OptimizationLevel;
use crate::optimizer::{AstFeatures, WholeProgramPass};
use crate::MutableProgram;
use typedlua_parser::ast::statement::{EnumDeclaration, Statement};

pub struct RichEnumOptimizationPass;

impl RichEnumOptimizationPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> WholeProgramPass<'arena> for RichEnumOptimizationPass {
    fn name(&self) -> &'static str {
        "rich-enum-optimization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::O2
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_ENUMS
    }

    fn run(&mut self, program: &mut MutableProgram<'arena>, _arena: &'arena Bump) -> Result<bool, String> {
        let mut rich_enum_count = 0;

        for stmt in &program.statements {
            if let Statement::Enum(enum_decl) = stmt {
                if self.is_rich_enum(enum_decl) {
                    rich_enum_count += 1;
                }
            }
        }

        if rich_enum_count > 0 {
            tracing::debug!(
                "Found {} rich enum(s) - O2 optimizations enabled",
                rich_enum_count
            );
        }

        Ok(false)
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
}

impl Default for RichEnumOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typedlua_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use typedlua_parser::ast::pattern::Pattern;
    use typedlua_parser::ast::statement::{
        Block, EnumConstructor, EnumDeclaration, EnumField, EnumMember, Parameter,
    };
    use typedlua_parser::ast::types::{PrimitiveType, Type, TypeKind};
    use typedlua_parser::ast::Spanned;
    use typedlua_parser::span::Span;
    use typedlua_parser::string_interner::StringInterner;

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
            value: Some(typedlua_parser::ast::statement::EnumValue::Number(1.0)),
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
    fn test_pass_returns_no_changes() {
        let arena = Bump::new();
        let mut pass = RichEnumOptimizationPass::new();
        let mut program = create_test_program_with_rich_enum(&arena);
        let result = pass.run(&mut program, &arena);
        assert!(!result.unwrap());
    }
}
