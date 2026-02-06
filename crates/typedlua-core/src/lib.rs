// Keep core-specific modules
pub mod cache;
pub mod codegen;
pub mod di;
pub mod optimizer;
pub mod type_checker;

// Re-export shared utilities and types used by typedlua-core
pub use typedlua_typechecker::{
    // Generic specialization optimizer functions
    build_substitutions,
    instantiate_function_declaration,
    // Module resolver
    module_resolver,
    // Symbol table for caching
    SerializableSymbolTable,
    Symbol,
    SymbolTable,
    // Main type checker
    TypeChecker,
};

// Re-export CLI modules - shared utilities
pub use typedlua_typechecker::cli::{config, diagnostics, errors, fs};

// Re-export common diagnostics for backward compatibility
pub use typedlua_typechecker::cli::diagnostics::{
    CollectingDiagnosticHandler, Diagnostic, DiagnosticHandler, DiagnosticLevel,
};

// Re-export parser types
pub use typedlua_parser::{
    ast::Program,
    string_interner::{CommonIdentifiers, StringId, StringInterner},
};

use std::path::PathBuf;

/// A module after parsing, before type checking.
/// Foundation for parallel parsing infrastructure.
pub struct ParsedModule {
    pub path: PathBuf,
    pub ast: Program,
    pub interner: StringInterner,
    pub common_ids: CommonIdentifiers,
    pub diagnostics: Vec<Diagnostic>,
}
