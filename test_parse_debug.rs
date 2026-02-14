use luanext_parser::lexer::Lexer;
use luanext_parser::parser::Parser;
use luanext_parser::cli::diagnostics::CollectingDiagnosticHandler;
use std::sync::Arc;

fn main() {
    let source = "function example(input: unknown): string {\n    if typeof(input) == \"string\" {\n        return input;\n    }\n    return \"\";\n}";
    
    let arena = bumpalo::Bump::new();
    let handler = Arc::new(CollectingDiagnosticHandler::new());
    let (interner, common) = luanext_parser::string_interner::StringInterner::new_with_common_identifiers();
    let mut lexer = Lexer::new(source, handler.clone(), &interner);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    let mut parser = Parser::new(tokens, handler.clone(), &interner, &common, &arena);
    match parser.parse() {
        Ok(program) => {
            println!("\nProgram parsed successfully!");
            println!("Number of statements: {}", program.statements.len());
            for (i, stmt) in program.statements.iter().enumerate() {
                println!("  Statement {}: {:?}", i, std::mem::discriminant(stmt));
            }
        }
        Err(e) => {
            println!("\nParse error: {:?}", e);
        }
    }
}
