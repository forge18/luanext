//! Tests for path alias codegen integration
//!
//! Verifies that when alias_require_map is set on CodeGenerator,
//! the generated Lua code emits the resolved require paths instead
//! of the raw alias import sources.

use luanext_core::codegen::{CodeGenerator, LuaTarget};
use luanext_parser::string_interner::StringInterner;
use luanext_parser::{Lexer, Parser};
use std::collections::HashMap;
use std::sync::Arc;

fn compile_with_alias_map(
    source: &str,
    alias_map: HashMap<String, String>,
) -> Result<String, String> {
    use luanext_core::diagnostics::CollectingDiagnosticHandler;
    use luanext_core::MutableProgram;
    use luanext_core::TypeChecker;
    use luanext_parser::diagnostics::CollectingDiagnosticHandler as ParserCollectingHandler;

    let arena = bumpalo::Bump::new();
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
    let _ = type_checker.check_program(&program);

    let mutable_program = MutableProgram::from_program(&program);
    let mut codegen = CodeGenerator::new(interner.clone())
        .with_target(LuaTarget::Lua54)
        .with_alias_require_map(alias_map);
    Ok(codegen.generate(&mutable_program))
}

#[test]
fn test_alias_import_rewritten_in_require() {
    let source = r#"import { add } from "@/utils""#;
    let mut alias_map = HashMap::new();
    alias_map.insert("@/utils".to_string(), "./src/utils".to_string());

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        result.contains(r#"require("./src/utils")"#),
        "Expected resolved path in require(), got: {}",
        result
    );
    assert!(
        !result.contains(r#"require("@/utils")"#),
        "Should not contain raw alias in require(), got: {}",
        result
    );
}

#[test]
fn test_alias_default_import_rewritten() {
    let source = r#"import Config from "@config/settings""#;
    let mut alias_map = HashMap::new();
    alias_map.insert(
        "@config/settings".to_string(),
        "./src/config/settings".to_string(),
    );

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        result.contains(r#"require("./src/config/settings")"#),
        "Expected resolved path, got: {}",
        result
    );
}

#[test]
fn test_alias_namespace_import_rewritten() {
    let source = r#"import * as utils from "@/shared/utils""#;
    let mut alias_map = HashMap::new();
    alias_map.insert("@/shared/utils".to_string(), "../shared/utils".to_string());

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        result.contains(r#"require("../shared/utils")"#),
        "Expected resolved path, got: {}",
        result
    );
}

#[test]
fn test_no_alias_map_preserves_raw_source() {
    let source = r#"import { add } from "./utils""#;
    let alias_map = HashMap::new(); // Empty map

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        result.contains(r#"require("./utils")"#),
        "Non-alias import should use raw source, got: {}",
        result
    );
}

#[test]
fn test_mixed_alias_and_relative_imports() {
    let source = r#"
import { add } from "@/utils"
import { sub } from "./local"
"#;
    let mut alias_map = HashMap::new();
    alias_map.insert("@/utils".to_string(), "./src/utils".to_string());

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        result.contains(r#"require("./src/utils")"#),
        "Alias import should be rewritten, got: {}",
        result
    );
    assert!(
        result.contains(r#"require("./local")"#),
        "Relative import should stay as-is, got: {}",
        result
    );
}

#[test]
fn test_type_only_alias_import_generates_no_require() {
    let source = r#"import type { MyType } from "@/types""#;
    let mut alias_map = HashMap::new();
    alias_map.insert("@/types".to_string(), "./src/types".to_string());

    let result = compile_with_alias_map(source, alias_map).unwrap();
    assert!(
        !result.contains("require"),
        "Type-only imports should generate no require(), got: {}",
        result
    );
}
