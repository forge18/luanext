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

// Re-exports from ecosystem crates - these replace local modules
pub use lua_sourcemap as sourcemap;
pub use typedlua_parser as parser_crate;
pub use typedlua_parser::ast;
pub use typedlua_parser::lexer;
pub use typedlua_parser::parser;
pub use typedlua_parser::span;
pub use typedlua_parser::string_interner;

pub use arena::Arena;
pub use ast::{Program, Spanned};
pub use codegen::CodeGenerator;
pub use config::{CliOverrides, CompilerConfig};
pub use di::Container;
pub use diagnostics::{
    error_codes, Diagnostic, DiagnosticCode, DiagnosticHandler, DiagnosticLevel,
    DiagnosticRelatedInformation, DiagnosticSuggestion,
};
pub use errors::CompilationError;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use span::Span;
pub use string_interner::{StringId, StringInterner};
pub use typechecker::{
    SerializableSymbol, SerializableSymbolTable, SymbolTable, TypeChecker, TypeEnvironment,
};
