//! Test compilation helpers for TypedLua
//!
//! Provides convenient functions for compiling TypedLua source code
//! in tests, using proper DI through the Container.

use luanext_core::codegen::{CodeGenerator, LuaTarget};
use luanext_core::config::{CompilerConfig, OptimizationLevel};
use luanext_core::di::DiContainer;
use luanext_core::diagnostics::{CollectingDiagnosticHandler, DiagnosticHandler};
use luanext_core::fs::MockFileSystem;
use luanext_core::optimizer::Optimizer;
use luanext_core::MutableProgram;
use luanext_core::TypeChecker;
use luanext_parser::string_interner::StringInterner;
use luanext_parser::{Lexer, Parser};
use std::sync::Arc;

/// Compile TypedLua source code without stdlib
///
/// # Arguments
/// * `source` - The TypedLua source code to compile
///
/// # Returns
/// The generated Lua code or an error message
pub fn compile(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile(source)
}

/// Compile TypedLua source code without stdlib and with optimization
///
/// # Arguments
/// * `source` - The TypedLua source code to compile
/// * `level` - The optimization level to apply
///
/// # Returns
/// The generated Lua code or an error message
pub fn compile_with_optimization(source: &str, level: OptimizationLevel) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_optimization(source, level)
}

/// Compile TypedLua source code with stdlib loaded
///
/// Use this for tests that need standard library features
/// like debug.traceback(), print(), etc.
///
/// # Arguments
/// * `source` - The TypedLua source code to compile
///
/// # Returns
/// The generated Lua code or an error message
pub fn compile_with_stdlib(source: &str) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib(source)
}

/// Compile TypedLua source code with stdlib loaded and optimization
///
/// Use this for tests that need both standard library features and optimization.
///
/// # Arguments
/// * `source` - The TypedLua source code to compile
/// * `level` - The optimization level to apply
///
/// # Returns
/// The generated Lua code or an error message
pub fn compile_with_stdlib_and_optimization(
    source: &str,
    level: OptimizationLevel,
) -> Result<String, String> {
    let config = CompilerConfig::default();
    let mut container = DiContainer::production(config);
    container.compile_with_stdlib_and_optimization(source, level)
}

/// Type check TypedLua source code
///
/// Returns the symbol table for further inspection, or an error message.
///
/// # Arguments
/// * `source` - The TypedLua source code to type check
///
/// # Returns
/// Ok(()) if type checking succeeds, or an error message
pub fn type_check(source: &str) -> Result<(), String> {
    // Use Box::leak for 'static arena in tests (acceptable for test helpers)
    let arena: &'static bumpalo::Bump = Box::leak(Box::new(bumpalo::Bump::new()));
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common_ids) = StringInterner::new_with_common_identifiers();
    let interner = std::rc::Rc::new(interner);

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexing failed: {:?}", e))?;

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common_ids, arena);
    let program = parser
        .parse()
        .map_err(|e| format!("Parsing failed: {:?}", e))?;

    let mut type_checker = TypeChecker::new(handler, &interner, &common_ids, arena);
    type_checker
        .check_program(&program)
        .map_err(|e| e.message)?;

    Ok(())
}

/// Create a test container with mock file system
///
/// Useful for tests that need to test file system interactions.
///
/// # Arguments
/// * `config` - Compiler configuration to use
///
/// # Returns
/// A DiContainer with mock file system
pub fn create_test_container(config: CompilerConfig) -> DiContainer {
    let diagnostics = Arc::new(CollectingDiagnosticHandler::new());
    let fs = Arc::new(MockFileSystem::new());
    DiContainer::test(config, diagnostics, fs)
}

/// Compile TypedLua source code targeting a specific Lua version
///
/// Use this for Lua version compatibility tests that need to verify the
/// generated output syntax differs by target (e.g. Lua51 uses `_bit_band()`
/// while Lua53+ uses native `&`).
///
/// # Arguments
/// * `source` - The TypedLua source code to compile
/// * `target` - The Lua version target for code generation
///
/// # Returns
/// The generated Lua code or an error message
pub fn compile_with_target(source: &str, target: LuaTarget) -> Result<String, String> {
    use bumpalo::Bump;
    use luanext_parser::diagnostics::CollectingDiagnosticHandler as ParserCollectingHandler;

    let arena = Bump::new();
    let parser_handler =
        Arc::new(ParserCollectingHandler::new()) as Arc<dyn luanext_parser::DiagnosticHandler>;
    let typecheck_handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common_ids) = StringInterner::new_with_common_identifiers();
    let interner = Arc::new(interner);

    let mut lexer = Lexer::new(source, parser_handler.clone(), &interner);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexing failed: {:?}", e))?;

    let mut parser = Parser::new(
        tokens,
        parser_handler.clone(),
        &interner,
        &common_ids,
        &arena,
    );
    let program = parser
        .parse()
        .map_err(|e| format!("Parsing failed: {:?}", e))?;

    let mut type_checker =
        TypeChecker::new(typecheck_handler.clone(), &interner, &common_ids, &arena);
    type_checker
        .check_program(&program)
        .map_err(|e| e.message)?;

    let mut mutable_program = MutableProgram::from_program(&program);

    let mut optimizer = Optimizer::new(
        OptimizationLevel::None,
        typecheck_handler.clone(),
        interner.clone(),
    );
    if let Err(err_msg) = optimizer.optimize(&mut mutable_program, &arena) {
        typecheck_handler.warning(
            luanext_parser::span::Span::dummy(),
            &format!("Optimization warning: {}", err_msg),
        );
    }

    let mut codegen = CodeGenerator::new(interner.clone()).with_target(target);
    Ok(codegen.generate(&mutable_program))
}
