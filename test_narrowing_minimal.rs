use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_typechecker::cli::diagnostics::CollectingDiagnosticHandler;
use luanext_typechecker::{TypeCheckError, TypeChecker};
use std::sync::Arc;

fn main() {
    // Build source with explicit newlines
    let source = "function example(x: unknown): unknown {\n    assertType<string>(x);\n    return x;\n}";

    println!("Source code:");
    println!("{}", source);
    println!("\n---\n");

    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) =
        luanext_parser::string_interner::StringInterner::new_with_common_identifiers();

    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    println!("Parsed {} statements", program.statements.len());

    let mut type_checker = TypeChecker::new(handler.clone(), &interner, &common, &arena);
    let result = type_checker.check_program(&program);

    match result {
        Ok(()) => println!("✅ Type checking passed!"),
        Err(e) => println!("❌ Type checking failed: {}", e),
    }
}
