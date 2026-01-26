pub mod arena;
pub mod cache;
pub mod codegen;
pub mod config;
pub mod di;
pub mod diagnostics;
pub mod errors;
pub mod fs;
pub mod module_resolver;
pub mod optimizer;
pub mod stdlib;
pub mod typechecker;

pub use lua_sourcemap as sourcemap;
pub use typedlua_parser::span::Span;
pub use typedlua_parser::string_interner::{CommonIdentifiers, StringId, StringInterner};

pub use arena::Arena;
pub use codegen::{CodeGenerator, LuaTarget};
pub use config::{CliOverrides, CompilerConfig};
pub use di::Container;
pub use diagnostics::{
    error_codes, CollectingDiagnosticHandler, Diagnostic, DiagnosticCode, DiagnosticHandler,
    DiagnosticLevel, DiagnosticRelatedInformation, DiagnosticSuggestion,
};
pub use errors::CompilationError;
pub use typechecker::{
    SerializableSymbol, SerializableSymbolTable, SymbolTable, TypeChecker, TypeEnvironment,
};
