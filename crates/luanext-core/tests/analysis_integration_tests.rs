//! Integration tests for the optimizer analysis infrastructure.
//!
//! Tests the full pipeline: parse LuaNext source → build MutableProgram →
//! compute AnalysisContext → verify analysis properties.

use luanext_core::optimizer::analysis::{
    AliasAnalyzer, AliasResult, AnalysisContext, BlockId, CfgBuilder, DominatorTree,
    MemoryLocation, SideEffectAnalyzer, Terminator,
};
use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
use luanext_parser::ast::pattern::Pattern;
use luanext_parser::ast::statement::{
    Block, FunctionDeclaration, IfStatement, Parameter, ReturnStatement, Statement,
    VariableDeclaration, VariableKind, WhileStatement,
};
use luanext_parser::ast::Ident;
use luanext_parser::span::Span;
use luanext_parser::string_interner::StringInterner;
use std::sync::Arc;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_ident(interner: &StringInterner, name: &str) -> Ident {
    Ident {
        node: interner.get_or_intern(name),
        span: Span::dummy(),
    }
}

fn make_expr_nil() -> Expression<'static> {
    Expression {
        kind: ExpressionKind::Literal(Literal::Nil),
        span: Span::dummy(),
        annotated_type: None,
        receiver_class: None,
    }
}

fn make_expr_true() -> Expression<'static> {
    Expression {
        kind: ExpressionKind::Literal(Literal::Boolean(true)),
        span: Span::dummy(),
        annotated_type: None,
        receiver_class: None,
    }
}

fn make_number(n: f64) -> Expression<'static> {
    Expression {
        kind: ExpressionKind::Literal(Literal::Number(n)),
        span: Span::dummy(),
        annotated_type: None,
        receiver_class: None,
    }
}

fn make_var_decl<'a>(interner: &StringInterner, name: &str) -> Statement<'a> {
    Statement::Variable(VariableDeclaration {
        kind: VariableKind::Local,
        pattern: Pattern::Identifier(make_ident(interner, name)),
        type_annotation: None,
        initializer: make_expr_nil(),
        span: Span::new(0, 10, 1, 1),
    })
}

fn empty_block() -> Block<'static> {
    Block {
        statements: &[],
        span: Span::dummy(),
    }
}

// ── CFG Integration Tests ───────────────────────────────────────────────────

#[test]
fn test_cfg_linear_program() {
    // local x = nil; local y = nil; local z = nil
    let interner = StringInterner::new();
    let stmts = vec![
        make_var_decl(&interner, "x"),
        make_var_decl(&interner, "y"),
        make_var_decl(&interner, "z"),
    ];

    let cfg = CfgBuilder::build(&stmts);

    // Should have: ENTRY, EXIT, code block = 3 blocks minimum
    assert!(
        cfg.block_count() >= 3,
        "Linear code needs at least 3 blocks"
    );

    // All 3 statements should map to the same code block
    let block_x = cfg.stmt_to_block.get(&0).unwrap();
    let block_y = cfg.stmt_to_block.get(&1).unwrap();
    let block_z = cfg.stmt_to_block.get(&2).unwrap();
    assert_eq!(
        block_x, block_y,
        "All linear statements should be in the same block"
    );
    assert_eq!(
        block_y, block_z,
        "All linear statements should be in the same block"
    );

    // No loop headers
    assert!(cfg.loop_headers.is_empty(), "No loops → no loop headers");
}

#[test]
fn test_cfg_if_else_diamond() {
    // local x = nil
    // if true then <body> else <body> end
    let interner = StringInterner::new();
    let stmts = vec![
        make_var_decl(&interner, "x"),
        Statement::If(IfStatement {
            condition: make_expr_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(10, 50, 2, 1),
        }),
    ];

    let cfg = CfgBuilder::build(&stmts);

    // Diamond structure: ENTRY, code, then, else, join, EXIT ≥ 5 blocks
    assert!(
        cfg.block_count() >= 5,
        "If/else diamond should have ≥ 5 blocks, got {}",
        cfg.block_count()
    );

    // The if statement block should have a Branch terminator
    let if_block_id = *cfg.stmt_to_block.get(&1).unwrap();
    let if_block = cfg.block(if_block_id).unwrap();
    assert!(
        matches!(if_block.terminator, Terminator::Branch { .. }),
        "If statement block should have Branch terminator"
    );
}

#[test]
fn test_cfg_while_loop_back_edge() {
    // while true do end
    let stmts = vec![Statement::While(WhileStatement {
        condition: make_expr_true(),
        body: empty_block(),
        span: Span::new(0, 30, 1, 1),
    })];

    let cfg = CfgBuilder::build(&stmts);

    // Should have loop header
    assert!(
        !cfg.loop_headers.is_empty(),
        "While loop should create a loop header"
    );

    // There should be a LoopBack terminator somewhere
    let has_loop_back = cfg
        .blocks
        .iter()
        .any(|b| matches!(b.terminator, Terminator::LoopBack(_)));
    assert!(has_loop_back, "While loop should create a LoopBack edge");
}

#[test]
fn test_cfg_reverse_postorder() {
    // Linear: ENTRY → code → EXIT
    let interner = StringInterner::new();
    let stmts = vec![make_var_decl(&interner, "x")];
    let cfg = CfgBuilder::build(&stmts);

    let rpo = cfg.reverse_postorder();

    // ENTRY should be first
    assert_eq!(rpo[0], BlockId::ENTRY, "ENTRY should be first in RPO");
}

// ── Dominance Integration Tests ─────────────────────────────────────────────

#[test]
fn test_dominance_entry_dominates_all() {
    let interner = StringInterner::new();
    let stmts = vec![make_var_decl(&interner, "x"), make_var_decl(&interner, "y")];

    let cfg = CfgBuilder::build(&stmts);
    let dom_tree = DominatorTree::build(&cfg);

    // ENTRY should dominate every block
    for block in &cfg.blocks {
        if block.id != BlockId::ENTRY {
            assert!(
                dom_tree.dominates(BlockId::ENTRY, block.id),
                "ENTRY should dominate B{}",
                block.id.0
            );
        }
    }
}

#[test]
fn test_dominance_if_else_frontiers() {
    // if true then <body> else <body> end
    // The join block should be in the dominance frontier of both branches
    let stmts = vec![Statement::If(IfStatement {
        condition: make_expr_true(),
        then_block: empty_block(),
        else_ifs: &[],
        else_block: Some(empty_block()),
        span: Span::new(0, 50, 1, 1),
    })];

    let cfg = CfgBuilder::build(&stmts);
    let dom_tree = DominatorTree::build(&cfg);

    // All blocks should have an immediate dominator (except ENTRY)
    for block in &cfg.blocks {
        if block.id != BlockId::ENTRY {
            assert!(
                dom_tree.immediate_dominator(block.id).is_some(),
                "B{} should have an idom",
                block.id.0
            );
        }
    }
}

// ── SSA Integration Tests ───────────────────────────────────────────────────

#[test]
fn test_ssa_variable_versioning() {
    // local x = nil; local y = nil
    // Both should get version 1
    let interner = StringInterner::new();
    let stmts = vec![make_var_decl(&interner, "x"), make_var_decl(&interner, "y")];

    let cfg = CfgBuilder::build(&stmts);
    let dom_tree = DominatorTree::build(&cfg);
    let ssa = luanext_core::optimizer::analysis::SsaForm::build(&cfg, &dom_tree, &stmts);

    let x_id = interner.get_or_intern("x");
    let y_id = interner.get_or_intern("y");

    // Both should be tracked
    assert!(ssa.all_variables.contains(&x_id));
    assert!(ssa.all_variables.contains(&y_id));

    // Each should have exactly 1 definition version
    assert_eq!(*ssa.version_counters.get(&x_id).unwrap(), 1);
    assert_eq!(*ssa.version_counters.get(&y_id).unwrap(), 1);
}

#[test]
fn test_ssa_reaching_definitions() {
    // local x = nil
    // After defining x, it should be reachable at version 1
    let interner = StringInterner::new();
    let stmts = vec![make_var_decl(&interner, "x")];

    let cfg = CfgBuilder::build(&stmts);
    let dom_tree = DominatorTree::build(&cfg);
    let ssa = luanext_core::optimizer::analysis::SsaForm::build(&cfg, &dom_tree, &stmts);

    let x_id = interner.get_or_intern("x");

    // Find the block containing the definition
    let def_block = cfg
        .blocks
        .iter()
        .find(|b| !b.statement_indices.is_empty())
        .unwrap();

    let reaching = ssa.reaching_def(def_block.id, x_id);
    assert!(reaching.is_some());
    assert_eq!(reaching.unwrap().version, 1);
}

// ── Side-Effect Analysis Integration Tests ──────────────────────────────────

#[test]
fn test_side_effects_pure_function() {
    // function add(a, b) return a + b end
    // Should be detected as pure
    let interner = Arc::new(StringInterner::new());
    let arena = bumpalo::Bump::new();

    let a_id = interner.get_or_intern("a");
    let b_id = interner.get_or_intern("b");

    let add_expr = Expression {
        kind: ExpressionKind::Binary(
            luanext_parser::ast::expression::BinaryOp::Add,
            arena.alloc(Expression {
                kind: ExpressionKind::Identifier(a_id),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            }),
            arena.alloc(Expression {
                kind: ExpressionKind::Identifier(b_id),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            }),
        ),
        span: Span::dummy(),
        annotated_type: None,
        receiver_class: None,
    };
    let ret_values = arena.alloc_slice_clone(&[add_expr]);
    let ret_stmt = Statement::Return(ReturnStatement {
        values: ret_values,
        span: Span::dummy(),
    });
    let body_stmts = arena.alloc_slice_clone(&[ret_stmt]);
    let body = Block {
        statements: body_stmts,
        span: Span::dummy(),
    };

    let params = arena.alloc_slice_clone(&[
        Parameter {
            pattern: Pattern::Identifier(make_ident(&interner, "a")),
            type_annotation: None,
            default: None,
            is_rest: false,
            is_optional: false,
            span: Span::dummy(),
        },
        Parameter {
            pattern: Pattern::Identifier(make_ident(&interner, "b")),
            type_annotation: None,
            default: None,
            is_rest: false,
            is_optional: false,
            span: Span::dummy(),
        },
    ]);

    let stmts = vec![Statement::Function(FunctionDeclaration {
        name: make_ident(&interner, "add"),
        type_parameters: None,
        parameters: params,
        return_type: None,
        throws: None,
        body,
        span: Span::dummy(),
    })];

    let analyzer = SideEffectAnalyzer::new(interner.clone());
    let info = analyzer.analyze(&stmts);

    let add_name = interner.get_or_intern("add");
    assert!(
        info.effects(add_name).unwrap().is_pure(),
        "add(a, b) should be pure"
    );
    assert!(
        info.pure_functions.contains(&add_name),
        "add should be in pure_functions set"
    );
}

#[test]
fn test_side_effects_builtin_purity() {
    // Known builtins should be classified as pure
    let interner = Arc::new(StringInterner::new());

    let analyzer = SideEffectAnalyzer::new(interner.clone());
    let info = analyzer.analyze(&[]);

    // Math functions
    assert!(info.is_pure(interner.get_or_intern("math.abs")));
    assert!(info.is_pure(interner.get_or_intern("math.floor")));
    assert!(info.is_pure(interner.get_or_intern("math.sqrt")));

    // String functions
    assert!(info.is_pure(interner.get_or_intern("string.len")));
    assert!(info.is_pure(interner.get_or_intern("string.sub")));

    // Utility functions
    assert!(info.is_pure(interner.get_or_intern("type")));
    assert!(info.is_pure(interner.get_or_intern("tostring")));
    assert!(info.is_pure(interner.get_or_intern("tonumber")));
}

// ── Alias Analysis Integration Tests ────────────────────────────────────────

#[test]
fn test_alias_primitives_no_alias() {
    // local x = 1; local y = 2 — primitives don't alias
    let interner = StringInterner::new();
    let stmts = vec![
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "x")),
            type_annotation: None,
            initializer: make_number(1.0),
            span: Span::dummy(),
        }),
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "y")),
            type_annotation: None,
            initializer: make_number(2.0),
            span: Span::dummy(),
        }),
    ];

    let analyzer = AliasAnalyzer::new();
    let info = analyzer.analyze(&stmts);

    let x_id = interner.get_or_intern("x");
    let y_id = interner.get_or_intern("y");

    assert_eq!(
        info.query(&MemoryLocation::Local(x_id), &MemoryLocation::Local(y_id)),
        AliasResult::NoAlias,
        "Primitives should not alias"
    );
}

#[test]
fn test_alias_table_assignment() {
    // local a = {}; local b = a — tables may alias
    let interner = StringInterner::new();
    let stmts = vec![
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "a")),
            type_annotation: None,
            initializer: Expression {
                kind: ExpressionKind::Object(&[]),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            },
            span: Span::dummy(),
        }),
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(make_ident(&interner, "b")),
            type_annotation: None,
            initializer: Expression {
                kind: ExpressionKind::Identifier(interner.get_or_intern("a")),
                span: Span::dummy(),
                annotated_type: None,
                receiver_class: None,
            },
            span: Span::dummy(),
        }),
    ];

    let analyzer = AliasAnalyzer::new();
    let info = analyzer.analyze(&stmts);

    let a_id = interner.get_or_intern("a");
    let b_id = interner.get_or_intern("b");

    assert_eq!(
        info.query(&MemoryLocation::Local(a_id), &MemoryLocation::Local(b_id)),
        AliasResult::MayAlias,
        "b = a (table) should create may-alias"
    );
}

// ── Full Pipeline Integration Tests ─────────────────────────────────────────

#[test]
fn test_full_analysis_pipeline() {
    // Smoke test: run the full AnalysisContext.compute() pipeline
    let interner = Arc::new(StringInterner::new());
    let arena = bumpalo::Bump::new();

    let params = arena.alloc_slice_clone(&[Parameter {
        pattern: Pattern::Identifier(make_ident(&interner, "n")),
        type_annotation: None,
        default: None,
        is_rest: false,
        is_optional: false,
        span: Span::dummy(),
    }]);

    let ret_values = arena.alloc_slice_clone(&[Expression {
        kind: ExpressionKind::Identifier(interner.get_or_intern("n")),
        span: Span::dummy(),
        annotated_type: None,
        receiver_class: None,
    }]);
    let body_stmts = arena.alloc_slice_clone(&[Statement::Return(ReturnStatement {
        values: ret_values,
        span: Span::dummy(),
    })]);

    let stmts: Vec<Statement<'_>> = vec![
        make_var_decl(&interner, "x"),
        Statement::Function(FunctionDeclaration {
            name: make_ident(&interner, "id"),
            type_parameters: None,
            parameters: params,
            return_type: None,
            throws: None,
            body: Block {
                statements: body_stmts,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }),
    ];

    let program = luanext_core::MutableProgram {
        statements: stmts,
        span: Span::dummy(),
    };
    let mut ctx = AnalysisContext::new();
    let result = ctx.compute(&program, interner.clone());
    assert!(result.is_ok(), "Analysis should succeed");

    // Top-level analysis should exist
    let top = ctx.top_level();
    assert!(top.is_some(), "Top-level analysis should exist");

    // Function analysis should exist
    let id_name = interner.get_or_intern("id");
    let func = ctx.function_analysis(id_name);
    assert!(func.is_some(), "Function 'id' should be analyzed");

    // Side effects should be computed
    let side_effects = ctx.side_effects();
    assert!(side_effects.is_some(), "Side effects should be computed");
}

#[test]
fn test_analysis_multiple_functions() {
    // Test analysis with multiple function declarations
    let interner = Arc::new(StringInterner::new());
    let arena = bumpalo::Bump::new();

    let empty_params: &[Parameter<'_>] = arena.alloc_slice_clone(&[]);
    let empty_body = Block {
        statements: &[],
        span: Span::dummy(),
    };

    let stmts: Vec<Statement<'_>> = vec![
        Statement::Function(FunctionDeclaration {
            name: make_ident(&interner, "foo"),
            type_parameters: None,
            parameters: empty_params,
            return_type: None,
            throws: None,
            body: empty_body,
            span: Span::dummy(),
        }),
        Statement::Function(FunctionDeclaration {
            name: make_ident(&interner, "bar"),
            type_parameters: None,
            parameters: empty_params,
            return_type: None,
            throws: None,
            body: Block {
                statements: &[],
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }),
    ];

    let program = luanext_core::MutableProgram {
        statements: stmts,
        span: Span::dummy(),
    };
    let mut ctx = AnalysisContext::new();
    let result = ctx.compute(&program, interner.clone());
    assert!(result.is_ok());

    // Both functions should be analyzed
    let analyzed = ctx.analyzed_functions();
    assert!(analyzed.len() >= 3, "Should have top-level + foo + bar");

    let foo_id = interner.get_or_intern("foo");
    let bar_id = interner.get_or_intern("bar");
    assert!(ctx.function_analysis(foo_id).is_some());
    assert!(ctx.function_analysis(bar_id).is_some());
}
