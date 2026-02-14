use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_typechecker::cli::diagnostics::CollectingDiagnosticHandler;
use std::sync::Arc;

fn main() {
    let source = r#"function example(x: unknown): unknown {
    assertType<string>(x);
    return x;
}"#;

    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) =
        luanext_parser::string_interner::StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");

    println!("Tokens:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, tok);
        if i > 20 { break; }
    }

    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    let program = parser.parse().expect("Parsing failed");

    println!("\nParsed {} statements", program.statements.len());
    for (i, stmt) in program.statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, std::mem::discriminant(stmt));
    }
}
