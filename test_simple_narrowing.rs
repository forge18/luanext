use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_typechecker::cli::diagnostics::CollectingDiagnosticHandler;
use luanext_typechecker::{TypeCheckError, TypeChecker};
use std::sync::Arc;

fn main() {
    let source = r#"
        function example(input: unknown): number {
            const x = input;
            return 42;
        }
    "#;

    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) =
        luanext_parser::string_interner::StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");
    let mut type_checker = TypeChecker::new(handler.clone(), &interner, &common, &arena);
    let result = type_checker.check_program(&program);

    match result {
        Ok(()) => println!("✅ Type checking succeeded!"),
        Err(err) => {
            println!("❌ Type checking failed: {}", err);
            for diag in handler.get_diagnostics() {
                println!("  - {}", diag.message);
            }
        }
    }
}
