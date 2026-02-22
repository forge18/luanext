# LSP Features

Completion, hover, definition, references, rename, code actions, signature help, and formatting.

## Overview

Each LSP feature is implemented as a module under `crates/luanext-lsp/src/features/`. Features are organized into four categories: navigation, edit, formatting, and structure.

## Navigation Features

### Go-to-Definition

**File**: `features/navigation/definition.rs`

Resolves the definition location for a symbol at the cursor position:

- Local variables → declaration site
- Function calls → function declaration
- Imported symbols → source module export
- Class members → member declaration in class
- Cross-file: follows imports to source modules

### Hover

**File**: `features/navigation/hover.rs`

Shows type information and documentation on hover:

- Variable type
- Function signature
- Class member info
- Type-only imports show "*Imported as type-only*" note

Cross-file hover resolves imported symbols via `provide_with_manager()`, which type-checks the source module and looks up the exported symbol's type. Shows "*Imported from `./module`*" annotation. Falls back to local-only hover (via trait `provide()`) when no `DocumentManager` is available. When the source module is not open in the editor, `DocumentManager::load_unopened_module()` reads the file from disk via `ModuleResolver::read_file()` and creates a temporary `Document` for type-checking.

### Find References

**File**: `features/navigation/references.rs`

Finds all locations where a symbol is used:

- Declarations
- Assignments
- Read accesses
- Cross-file: follows import/export chains and re-export chains via SymbolIndex

## Edit Features

### Completion

**File**: `features/edit/completion.rs`

Provides autocomplete suggestions:

- Local variables in scope
- Function parameters
- Class members (respecting access modifiers)
- Imported symbols
- Keywords
- Type names (for type annotations)

Type-only imports show "(type-only import)" suffix in completion items.

Helper: `get_type_only_imports()` collects type-only imported names for annotation.

Cross-file completion resolves imported symbols via `DocumentManager` — loads the source module, type-checks it, and extracts members from the actual type. Shared import scanning logic lives in `features/import_utils.rs`.

### Rename

**File**: `features/edit/rename.rs`

Safe symbol renaming:

- Renames the declaration and all references
- Cross-file: renames across import/export boundaries
- Validates the new name is valid

### Code Actions

**File**: `features/edit/code_actions.rs`

Quick fixes and refactoring actions:

- Add missing imports
- Fix type errors
- Remove unused variables

### Signature Help

**File**: `features/edit/signature_help.rs`

Shows function parameter information as you type:

- Parameter names and types
- Current parameter highlighting
- Overload resolution

## Formatting

**File**: `features/formatting/formatting.rs`

Document formatting:

- Full document formatting
- Range formatting
- On-type formatting

## Structure Features

### Document Symbols

**File**: `features/structure/symbols.rs`

Provides document outline:

- Functions, classes, interfaces, enums
- Nested class members
- Hierarchical symbol tree

Handles `VariableKind::Global` for global variable declarations.

### Folding Ranges

**File**: `features/structure/folding_range.rs`

Code folding regions:

- Function bodies
- Class bodies
- If/while/for blocks
- Import groups

### Selection Range

**File**: `features/structure/selection_range.rs`

Expand/shrink selection intelligently:

- Word → expression → statement → block → function

## Cross-References

- [LSP Architecture](lsp-architecture.md) — server setup and message routing
- [LSP Analysis](lsp-analysis.md) — symbol index powering these features
