# LuaNext Architecture

LuaNext is a Rust-based compiler that transpiles a TypeScript-inspired language (`.luax` files) to Lua, targeting 6 Lua versions (5.1, 5.2, 5.3, 5.4, 5.5, LuaJIT). It provides a full-featured type system, class hierarchy, module system, optimizer, incremental compilation, and LSP-based IDE support.

## Crate Map

| Crate | Purpose | Spec(s) |
| ----- | ------- | ------- |
| `luanext-parser` | Lexer, parser, AST definitions, incremental parsing, string interner | [lexer](systems/language/lexer-and-tokens.md), [parser](systems/language/parser.md), [ast](systems/language/ast.md) |
| `luanext-typechecker` | Type checking (5 phases), module registry, symbol tables, config | [type-checking](systems/language/type-checking.md), [type-compatibility](systems/language/type-compatibility.md), [module-resolution](systems/features/module-resolution.md), [config](systems/tooling/config.md) |
| `luanext-core` | Code generation, optimizer (26+ passes), cache system, DI container | [codegen-*](systems/compiler/), [optimizer-*](systems/compiler/), [incremental-cache](systems/compiler/incremental-cache.md) |
| `luanext-lsp` | Language Server Protocol: completion, hover, go-to-def, rename, etc. | [lsp-*](systems/tooling/) |
| `luanext-cli` | CLI entry point, config loading, compilation pipeline orchestration | [cli](systems/tooling/cli.md) |
| `luanext-runtime` | Lua code snippets embedded in codegen output (class, enum, decorator, reflection) | [runtime-library](systems/compiler/runtime-library.md) |
| `luanext-sourcemap` | Source map generation (V3 spec, VLQ encoding) | [sourcemap](systems/compiler/sourcemap.md) |
| `luanext-test-helpers` | Shared test infrastructure: `compile()`, `execute_and_get()`, mocks | (internal) |

### Dependency Graph

```text
luanext-cli ──> luanext-core ──> luanext-parser
                     │                │
                     │                └──> luanext-sourcemap
                     │
                     └──> luanext-typechecker
                     │
                     └──> luanext-runtime

luanext-lsp ──> luanext-parser
      │
      └──> luanext-typechecker

luanext-test-helpers ──> luanext-core, luanext-parser, luanext-lsp
```

## Compilation Pipeline

```text
Source Files (.luax)
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│  1. CONFIG LOADING                                      │
│     CLI args + luanext.config.yaml → CompilerConfig      │
│     Glob pattern expansion → file list                  │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  2. PARALLEL PARSING  (rayon)                           │
│     Each file: Lexer → Parser → Program<'arena>         │
│     Arena-allocated AST per file                        │
│     Result: Vec<ParsedModule>                           │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  3. MODULE REGISTRATION                                 │
│     ImportScanner discovers dependencies                │
│     Topological sort (follows TypeOnly edges)           │
│     ModuleRegistry.register_parsed() for each module    │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  4. TYPE CHECKING  (incremental, parallel)              │
│     Cache check → skip if unchanged                     │
│     5 phases: declaration → checking → module →         │
│               inference → validation                    │
│     Result: type-annotated AST + module exports         │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  5. MODULE GRAPH  (O2+ only)                            │
│     Build dependency graph for LTO                      │
│     Track exports, imports, re-exports per module       │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  6. OPTIMIZATION  (O0-O3, fixed-point)                  │
│     Program → MutableProgram                            │
│     Apply passes by level (max 10 iterations)           │
│     LTO passes at O2+ (dead imports/exports, unused     │
│     modules, re-export flattening)                      │
└──────────────────────────┬──────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  7. CODE GENERATION  (parallel, target-specific)        │
│     CodeGenStrategy selects Lua target behavior         │
│     Tree shaking, scope hoisting                        │
│     Emit Lua code (readable / compact / minified)       │
│     Optional source map generation                      │
└──────────────────────────┬──────────────────────────────┘
                           ▼
              Output (.lua files + optional .map)
```

`[NOTE: BEHAVIOR]` LTO passes run during the codegen phase (step 7), not during the optimizer phase (step 6), because they transform the AST inside the codegen closure where `Vec<Statement<'arena>>` is available.

`[NOTE: BEHAVIOR]` Incremental compilation can skip steps 2-7 entirely for unchanged modules via the cache system. Cache hits use serialized exports from `.luanext-cache/`.

## Memory Model

- **Arena allocation**: All AST nodes use `bumpalo::Bump` with lifetime `'arena`. Bulk O(1) deallocation when the arena drops.
- **StringInterner**: Thread-safe `ThreadedRodeo` maps strings to `StringId` values. Shared across parallel compilation via `Arc`. Serializable via `from_strings()` / `to_strings()`.
- **MutableProgram**: The optimizer converts immutable `Program<'arena>` (with `&'arena [Statement]`) to `MutableProgram<'arena>` (with `Vec<Statement>`) for in-place transformation, then converts back.

`[NOTE: UNSAFE]` `ModuleRegistry` uses `unsafe transmute` to convert `Symbol<'arena>` to `Symbol<'static>` for cross-module symbol sharing. See `symbol_to_static` in `module_phase.rs`.

## Spec Index

### Language Frontend

Syntax, parsing, and the type system.

- [Lexer and Tokens](systems/language/lexer-and-tokens.md) — Tokenization, 72 keywords, string interning, spans
- [Parser](systems/language/parser.md) — Recursive descent parser, arena allocation, error recovery, block syntax
- [AST](systems/language/ast.md) — Statement, Expression, Type, and Pattern node hierarchies
- [Type Primitives](systems/language/type-primitives.md) — Primitive types, literals, unions, intersections, nullable, arrays, tuples
- [Type Advanced](systems/language/type-advanced.md) — Generics, conditional types, mapped types, keyof, template literals, infer
- [Type Checking](systems/language/type-checking.md) — 5-phase type checker, symbol tables, inference, narrowing, stdlib
- [Type Compatibility](systems/language/type-compatibility.md) — Assignability rules, structural typing, cycle detection

### Language Features

High-level features that span parser, type checker, and codegen.

- [Classes](systems/features/classes.md) — Class declarations, inheritance, access control, primary constructors
- [Class Advanced](systems/features/class-advanced.md) — Operator overloading, decorators, getters/setters, abstract/final
- [Modules](systems/features/modules.md) — Import/export syntax, re-exports, type-only imports
- [Module Resolution](systems/features/module-resolution.md) — Module registry, resolver, dependency graph, path aliases
- [Enums](systems/features/enums.md) — Simple and rich enums with fields, methods, interfaces
- [Pattern Matching](systems/features/pattern-matching.md) — Match expressions, pattern kinds, guards, destructuring

### Compiler Backend

Code generation, optimization, caching, and runtime support.

- [Codegen Architecture](systems/compiler/codegen-architecture.md) — CodeGenerator, Emitter, builder, output formats
- [Codegen Statements](systems/compiler/codegen-statements.md) — Variable, function, control flow, try/catch codegen
- [Codegen Expressions](systems/compiler/codegen-expressions.md) — Operators, calls, optional chaining, pipe, ternary, templates
- [Codegen Targets](systems/compiler/codegen-targets.md) — CodeGenStrategy trait, 6 Lua targets, version-specific behavior
- [Optimizer Architecture](systems/compiler/optimizer-architecture.md) — Visitor traits, pass registration, O0-O3, fixed-point loop
- [Optimizer Passes](systems/compiler/optimizer-passes.md) — All 26+ individual passes with behavior descriptions
- [Optimizer Analysis](systems/compiler/optimizer-analysis.md) — CFG, dominance trees, SSA, alias analysis, side effects
- [Optimizer LTO](systems/compiler/optimizer-lto.md) — Module graph, dead import/export elimination, re-export flattening
- [Incremental Parsing](systems/compiler/incremental-parsing.md) — `parse_incremental()`, multi-arena, statement caching
- [Incremental Cache](systems/compiler/incremental-cache.md) — CacheManager, manifest, serializable types, invalidation
- [Runtime Library](systems/compiler/runtime-library.md) — Lua runtime snippets for class, enum, decorator, reflection, bitwise
- [Source Maps](systems/compiler/sourcemap.md) — V3 spec, VLQ encoding, builder, position translation

### Tooling

Developer-facing tools: CLI, configuration, and IDE support.

- [CLI](systems/tooling/cli.md) — CLI flags, compilation pipeline orchestration, watch mode
- [Config](systems/tooling/config.md) — `luanext.config.yaml`, CompilerConfig, CompilerOptions
- [LSP Architecture](systems/tooling/lsp-architecture.md) — Server, message handler, DI, document management
- [LSP Features](systems/tooling/lsp-features.md) — Completion, hover, definition, references, rename, code actions
- [LSP Analysis](systems/tooling/lsp-analysis.md) — Symbol index, cross-file support, semantic tokens, inlay hints
