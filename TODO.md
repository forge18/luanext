# LuaNext TODO

## Dead Code Cleanup

- [x] Delete duplicate `crates/luanext-lsp/src/analysis/` directory (~1,592 lines) — superseded by `core/analysis/`
- [x] Delete unused `crates/luanext-lsp/src/features/semantic/semantic_lexeme.rs` (~1,092 lines)
- [x] Delete template `crates/luanext-typechecker/src/utils/narrowing_integration.rs` (~375 lines)
- [x] Delete dead type checker convenience wrappers: `validate_interface_members()`, `validate_index_signature()`, `has_circular_inheritance()`, `statement_always_returns()`
- [x] Delete dead DI container methods: removed `#[allow(dead_code)]` (methods kept — used in tests)
- [x] Delete dead semantic token helpers: `create_token()`, `classify_token()`
- [x] Delete dead LSP code: removed `#[allow(dead_code)]` from `BasicMessageHandler`, `MessageHandler::with_container()`, `DocumentManagerTrait`, etc.
- [x] Remove dead struct fields: `DevirtualizationPass.interner`, `Optimizer.handler`, `DocumentManager.module_registry`
- [x] Delete `apply_type_arguments()` stub, removed `#[allow(dead_code)]` from `TypeCheckVisitor::name()`
- [x] Fix `#[allow(dead_code)]` annotations: removed from all non-test items, added `#![allow(dead_code)]` to LSP binary (shared module tree)

## Features to Implement

- [x] Wire inlay hints expression collector — `collect_hints_from_expression()` now called from all statement types
- [x] Implement inlay hints with inferred types — looks up types via `TypeChecker::lookup_symbol()`
- [x] Add `completionItem/resolve` handler — `ResolveCompletionItem` handled in `message_handler.rs`
- [x] Implement completion resolve logic — generates markdown documentation from item kind/detail
