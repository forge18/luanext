// Keep core-specific modules
pub mod cache;
pub mod codegen;
pub mod di;
pub mod optimizer;
pub mod type_checker;

// Re-export shared utilities and types used by typedlua-core
pub use typedlua_typechecker::{
    // Modules - shared utilities
    config,
    diagnostics,
    errors,
    fs,
    module_resolver,
    // Generic specialization optimizer functions
    build_substitutions,
    instantiate_function_declaration,
    // Symbol table for caching
    SerializableSymbolTable,
    Symbol,
    SymbolTable,
    // Main type checker
    TypeChecker,
};

// Re-export common diagnostics for backward compatibility
pub use typedlua_typechecker::diagnostics::{
    CollectingDiagnosticHandler, Diagnostic, DiagnosticHandler, DiagnosticLevel,
};
