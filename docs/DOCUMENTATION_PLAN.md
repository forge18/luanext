# Documentation Revamp Plan

## Overview

LuaNext's documentation is being restructured into two clear locations:

- **`docs/`** — Contributor documentation (architecture, implementation patterns, design specs)
- **`docs-site/src/`** — User-facing documentation (guides, language reference, API reference, examples)

All documentation is rewritten from scratch by auditing the source code. Nothing is assumed accurate from existing docs.

### Naming Convention

All files use `kebab-case.md` (lowercase, hyphens). No exceptions.

### Documentation Standard

User-facing docs follow the [Diataxis](https://diataxis.fr/) framework:

- **Tutorials** — Learning-oriented (getting-started/)
- **How-to Guides** — Task-oriented (guides/)
- **Reference** — Information-oriented (reference/)
- **Explanation** — Understanding-oriented (language/)

Contributor docs follow: architecture → implementation → design.

---

## Contributor Documentation (`docs/`)

### Final Structure

```text
docs/
├── contributing.md          # PR workflow, code review, commit conventions
├── development-setup.md     # Local dev environment, tools, debugging
├── architecture.md          # System architecture overview
├── implementation.md        # Implementation patterns and conventions
├── testing.md               # Testing strategy, coverage, test patterns
├── performance.md           # Profiling, benchmarking, optimization
├── security.md              # Security considerations and threat model
│
├── compiler/
│   ├── parser.md           # Parser architecture, adding syntax
│   ├── typechecker.md      # Type system internals, type resolution
│   ├── optimizer.md        # Optimization passes, adding new optimizations
│   └── codegen.md          # Code generation, Lua target differences
│
├── module-system/
│   ├── resolution.md       # Module resolution algorithm
│   └── type-exports.md     # Cross-file type resolution
│
└── lsp/
    ├── features.md         # LSP feature implementation guide
    └── caching.md          # LSP caching and performance
```

14 files total (down from 25, excludes temporary design specs).

### Core Documentation

#### `contributing.md`

Contributor onboarding and workflow:

- PR workflow and branch conventions
- Code review process and expectations
- Commit message conventions
- Issue triage and bug reporting
- Documentation requirements
- Release process overview

#### `development-setup.md`

Local development environment:

- Prerequisites (Rust, tools, IDE setup)
- Building from source
- Running tests locally
- Debugging techniques (lldb/gdb, logging, tracing)
- Common development tasks
- Troubleshooting common issues

#### `architecture.md`

High-level system architecture:

- Workspace and crate structure
- Compiler pipeline overview: parse → type check → optimize → codegen
- Arena allocation pattern (`bumpalo::Bump`, `Type<'arena>` lifetimes)
- String interner (`ThreadedRodeo`) and symbol handling
- Cache and incremental compilation architecture
- Data flow between compilation phases
- Key architectural decisions and trade-offs

#### `implementation.md`

Implementation patterns and conventions:

- Rust coding standards and idioms
- Error handling patterns (`Result<T, E>`)
- Trait-based dependency injection for testability
- Visitor pattern usage (ExprVisitor, StmtVisitor, BlockVisitor)
- Arena-allocated AST patterns
- Type safety patterns and lifetime management
- Common pitfalls and how to avoid them

#### `testing.md`

Testing strategy and practices:

- Testing philosophy (unit vs integration, coverage targets)
- Unit testing patterns (`#[cfg(test)]` in same file)
- Integration testing structure (`tests/` directory)
- Test data management
- Snapshot testing for parser/codegen
- Property-based testing where applicable
- Running and debugging tests
- Coverage measurement with `cargo tarpaulin`

#### `performance.md`

Performance optimization guide:

- Profiling methodology (flamegraph, perf, Instruments)
- Benchmarking with criterion
- Memory profiling (heaptrack, valgrind)
- Cache effectiveness metrics
- Performance targets and SLOs
- Common performance bottlenecks
- Optimization case studies

#### `security.md`

Security considerations:

- Threat model for a compiler
- Input validation and parser safety
- Code generation safety (avoiding injection vulnerabilities)
- Dependency security and supply chain
- Sandboxing considerations
- Security review process

### Compiler Subsystems

#### `compiler/parser.md`

Parser internals and extension:

- Lexer architecture and token stream
- Recursive descent parsing strategy
- Error recovery mechanisms
- Span tracking for diagnostics
- How to add new syntax
- AST node design patterns
- Testing parser changes

#### `compiler/typechecker.md`

Type system internals:

- Type representation (`Type<'arena>` hierarchy)
- Type inference algorithm
- Constraint solving
- Generic type instantiation
- Subtyping and type compatibility
- Cross-file type resolution (module registry)
- How to add new type kinds
- Type serialization for caching

#### `compiler/optimizer.md`

Optimization pass architecture:

- Pass organization (composite vs standalone)
- Visitor-based transformations
- Four composite traversals (expr, elimination, data structure, function)
- Standalone passes (global localization, loop opt, rich enum opt, devirtualization, generic specialization)
- How to add new optimization passes
- Optimization correctness and testing
- Performance measurement

#### `compiler/codegen.md`

Code generation:

- Lua target abstraction (5.1, 5.2, 5.3, 5.4)
- Target-specific code generation strategies
- Source map generation
- Output formatting and minification
- Tree shaking and scope hoisting
- Testing codegen output

### Module System

#### `module-system/resolution.md`

Module resolution algorithm:

- Import statement processing
- Module path resolution (file system, modulePaths config)
- Module registry architecture
- Circular dependency detection (type-only vs value dependencies)
- Re-export chain handling
- Cache integration

#### `module-system/type-exports.md`

Cross-file type resolution:

- Exporting and importing types
- Type-only imports
- Symbol transmission across module boundaries
- Type compatibility checking across modules
- Handling incomplete type information during compilation

### LSP Subsystem

#### `lsp/features.md`

LSP feature implementation:

- LSP protocol overview and capabilities
- Document lifecycle management
- Feature implementation patterns (hover, completion, go-to-definition, etc.)
- Symbol indexing architecture
- Cross-file queries and workspace symbols
- Diagnostic generation and publishing
- Testing LSP features

#### `lsp/caching.md`

LSP caching and performance:

- Generic cache framework (VersionedCache, DocumentCache)
- Cache invalidation strategies (full, partial, cascade)
- Cross-file dependency tracking
- Cache memory management and LRU eviction
- Performance metrics and monitoring
- **Future:** Incremental parsing (statement-level caching, dirty regions, arena handling)

### `designs/`

Design specifications kept as reference:

| File | Content |
| ---- | ------- |
| language-spec.md | Canonical type system and language specification (4,400+ lines) |
| ast-structure.md | Complete AST node definitions |
| grammar.md | Formal EBNF grammar |
| lsp-design.md | LSP feature implementation spec |
| additional-features.md | Advanced features design (exception handling, decorators, etc.) |
| runtime-validation.md | Runtime validation system design |

---

## User-Facing Documentation (`docs-site/src/`)

### Structure

```text
docs-site/
└── src/
    ├── SUMMARY.md                   ✅ COMPLETE
    ├── introduction.md              ✅ COMPLETE
    │
    ├── getting-started/
    │   ├── installation.md          ✅ COMPLETE
    │   ├── quick-start.md           ✅ COMPLETE
    │   ├── editor-setup.md          ✅ COMPLETE
    │   └── project-setup.md         ✅ COMPLETE
    │
    ├── language/
    │   ├── basics.md                ✅ COMPLETE
    │   ├── control-flow.md          ✅ COMPLETE
    │   ├── functions.md             ✅ COMPLETE
    │   ├── type-system.md           ✅ COMPLETE
    │   ├── classes.md               ✅ COMPLETE
    │   ├── interfaces.md            ✅ COMPLETE
    │   ├── enums.md                 ✅ COMPLETE
    │   ├── modules.md               ✅ COMPLETE
    │   ├── error-handling.md        ✅ COMPLETE
    │   ├── pattern-matching.md      ✅ COMPLETE
    │   ├── decorators.md            ✅ COMPLETE
    │   ├── advanced-types.md        ✅ COMPLETE
    │   └── operators.md             ✅ COMPLETE
    │
    ├── guides/
    │   ├── migrating-from-lua.md    ✅ COMPLETE
    │   └── lua-targets.md           ✅ COMPLETE
    │
    └── reference/
        ├── cli.md                   ✅ COMPLETE
        ├── configuration.md         ✅ COMPLETE
        ├── standard-library.md      ✅ COMPLETE
        ├── utility-types.md         ✅ COMPLETE
        ├── reflection.md            ✅ COMPLETE
        ├── error-codes.md           ✅ COMPLETE
        ├── grammar.md               ✅ COMPLETE
        └── keywords.md              ✅ COMPLETE
```

**Status:** 25 content files + SUMMARY.md — ALL COMPLETE ✅

### `introduction.md`

Project overview, value proposition, and the LuaNext ecosystem.

#### Ecosystem

LuaNext is part of a broader ecosystem of Lua developer tools. Each tool works standalone on plain Lua but gains capabilities when used together.

**[Depot](https://github.com/forge18/depot)** — Lua Package Manager

- Local, project-scoped dependency management (like npm/cargo for Lua)
- Lua version manager (manage 5.1, 5.3, 5.4 installs)
- Lockfile support for reproducible builds
- SemVer version resolution
- LuaRocks compatible upstream package source
- Supply chain security (BLAKE3 checksums, sandboxed builds)

**[Lintomatic](https://github.com/forge18/lintomatic)** — Linter for Lua and LuaNext

- 100+ built-in rules across 8 categories
- Rust backend for performance
- Lua plugin API for custom rules
- Type-aware linting when used with LuaNext
- Auto-fix support
- Multiple presets and output formats

**[Canary](https://github.com/forge18/canary)** — Test Framework

- Rust-powered test runner with Jest-inspired API
- Built-in mocking and spying
- Snapshot testing
- Coverage collection
- Parallel test execution
- Works with both plain Lua and LuaNext

**[Wayfinder](https://github.com/forge18/wayfinder)** — Debugger (DAP)

- Debug Adapter Protocol implementation for Lua
- Works with PUC Lua 5.1-5.4 and LuaNext
- Source map support for LuaNext debugging
- Breakpoints, stepping, stack inspection, variable watches
- IDE integration (VS Code, Neovim, JetBrains)

### Core Libraries

These standalone crates are available for building ecosystem tools:

**[luanext-parser](https://github.com/forge18/luanext-parser)** — Parser Library

- Lexer and parser for LuaNext (.luax) files
- Arena-allocated AST (`bumpalo::Bump`)
- String interner for efficient symbol handling
- Full error recovery and span tracking

**[luanext-typechecker](https://github.com/forge18/luanext-typechecker)** — Type Checker Library

- Complete type checking for LuaNext
- Type inference, generics, conditional types
- Diagnostic reporting
- Module resolution and dependency tracking

**[luanext-sourcemap](https://github.com/forge18/luanext-sourcemap)** — Source Map Library

- Source map generation and consumption
- Maps compiled Lua positions back to LuaNext source
- Used by Wayfinder for debug source mapping

**[luanext-lsp](https://github.com/forge18/luanext-lsp)** — Language Server Protocol

- Full LSP implementation for LuaNext
- Completion, hover, go-to-definition, references, rename, formatting, diagnostics
- Semantic tokens, inlay hints, code actions
- Powers the VS Code extension

These crates are published separately so ecosystem tools can depend on them without pulling in the full compiler.

### Getting Started

| Page | Content |
| ---- | ------- |
| installation.md | Prerequisites (Rust), install, verify |
| quick-start.md | First .luax file, compile, run, see types catch a bug |
| editor-setup.md | VS Code extension, LSP features |
| project-setup.md | luanext.config.yaml, project structure, multi-file projects |

### Language Reference

Every page includes LuaNext source and compiled Lua output examples.

| Page | Key Features |
| ---- | ------------ |
| basics.md | `const`/`local`, primitive types (nil, boolean, number, integer, string, unknown, never, void, table, thread), type annotations, string templates |
| control-flow.md | if/elseif/else, while, for numeric, for generic, repeat/until, break, continue, labels/goto, blocks |
| functions.md | Declarations, arrow functions, generics, default/optional/rest params, throws clause, multiple return values |
| type-system.md | Unions, intersections, generics (constraints, defaults), type aliases, literal types, nullable, type narrowing |
| classes.md | Abstract/final, primary constructors, access modifiers, static/readonly, getters/setters, operator overloading (24 ops), inheritance, implements |
| interfaces.md | Property/method/index signatures, readonly, optional, structural typing, default implementations, extends |
| enums.md | Rich enums: string/number values, fields, constructors, methods, implements |
| modules.md | Import (default, named, namespace, type-only, mixed), export (declaration, named, default), re-exports, namespaces, module resolution, circular dependencies |
| error-handling.md | try/catch/finally, typed catches, multi-typed catches, throw, rethrow, try expressions, error chain `!!`, throws clause |
| pattern-matching.md | Match expressions, identifier/literal/array/object/wildcard/or patterns, guards, exhaustiveness |
| decorators.md | @decorator syntax, targets (class/method/property/getter/setter/operator), built-in decorators (@readonly, @sealed, @deprecated) |
| advanced-types.md | Conditional, mapped (+/-readonly/optional modifiers), template literal, keyof, typeof, infer, index access, type predicates, variadic |
| operators.md | Optional chaining, null coalesce, pipe, ternary, instanceof, error chain, type assertion, spread/rest, binary (23), unary (4), compound assignment (13) |

### Guides

| Page | Content |
| ---- | ------- |
| migrating-from-lua.md | Side-by-side Lua vs LuaNext comparisons |
| lua-targets.md | Lua 5.1-5.4 differences, choosing a target |

### Reference

| Page | Content |
| ---- | ------- |
| cli.md | Every command and flag |
| configuration.md | Every luanext.config.yaml option with types and defaults |
| standard-library.md | 24 global functions + 9 namespaces (coroutine, debug, io, math, os, package, string, table, utf8) with full signatures |
| utility-types.md | All 12 built-in utility types (Partial, Required, Readonly, Record, Pick, Omit, Exclude, Extract, NonNilable, Nilable, ReturnType, Parameters) |
| reflection.md | Reflect API (7 functions), field flags, type codes, metadata format |
| error-codes.md | Error/warning code catalog with fix suggestions |
| grammar.md | Formal EBNF grammar |
| keywords.md | All 64 keywords with categories and examples |

---

## Content Template

Every language page follows this structure:

```text
# Feature Name

One-paragraph summary.

## Syntax

[Code block with syntax placeholders]

## Examples

### [Realistic scenario name]

[LuaNext source]

Compiles to:

[Generated Lua output]

### [Another scenario]
...

## Details

Behavior, edge cases, interactions with other features.

## See Also

Links to related pages.
```

---

## Source of Truth

Every document is verified against actual source code.

| Topic | Source File |
| ----- | ---------- |
| CLI flags | `crates/luanext-cli/src/main.rs` |
| Config options | `crates/luanext-typechecker/src/cli/config.rs` |
| Types (21 kinds) | `crates/luanext-parser/src/ast/types.rs` |
| Statements (28 kinds) | `crates/luanext-parser/src/ast/statement.rs` |
| Expressions (28 kinds) | `crates/luanext-parser/src/ast/expression.rs` |
| Patterns (6 kinds) | `crates/luanext-parser/src/ast/pattern.rs` |
| Keywords (64) | `crates/luanext-parser/src/lexer/lexeme.rs` |
| Code generation | `crates/luanext-core/src/codegen/` |
| Optimizer | `crates/luanext-core/src/optimizer/mod.rs` |
| LSP capabilities | `crates/luanext-lsp/src/message_handler.rs` |
| Standard library | `crates/luanext-core/src/stdlib/lua54.d.luax` |
| Utility types (12) | `crates/luanext-typechecker/src/types/utility_types.rs` |
| Reflection API | `crates/luanext-runtime/src/reflection.rs` |
| Cache system | `crates/luanext-core/src/cache/` |
| Tree shaking | `crates/luanext-core/src/codegen/tree_shaking.rs` |
| Scope hoisting | `crates/luanext-core/src/codegen/scope_hoisting.rs` |
| Source maps | `crates/luanext-sourcemap/` |

---

## Execution Order

### Phase 1 — Clean up `docs/` ✅ COMPLETE

Deleted 16 stale files, renamed 9 to kebab-case.

### Phase 2 — User-facing docs (`docs-site/src/`) ✅ COMPLETE

All 25 content files created:

1. ✅ introduction.md
2. ✅ getting-started/ (installation, quick-start, editor-setup, project-setup)
3. ✅ language/ core (basics, control-flow, functions, type-system)
4. ✅ language/ OOP (classes, interfaces, enums, modules)
5. ✅ language/ advanced (error-handling, pattern-matching, decorators, advanced-types, operators)
6. ✅ guides/ (migrating-from-lua, lua-targets)
7. ✅ reference/ (cli, configuration, standard-library, utility-types, reflection, error-codes, grammar, keywords)
8. ✅ SUMMARY.md

**Key Corrections Applied:**

- Removed all LuaJIT references (LuaNext does NOT support LuaJIT)
- Removed enableOop/enableFp config options (features enabled by default)
- Changed all .luanext extensions to .luax (correct file extension)

### Phase 3 — Contributor docs (`docs/`) — PENDING

**Core Documentation (7 files):**

- contributing.md - PR workflow, code review, commit conventions
- development-setup.md - Local dev environment, debugging, tools
- architecture.md - System architecture overview (rewrite from code audit)
- implementation.md - Implementation patterns and conventions (rewrite from code audit)
- testing.md - Testing strategy, patterns, coverage
- performance.md - Profiling, benchmarking, optimization workflow
- security.md - Threat model, input validation, dependency security (rewrite from code audit)

**Compiler Subsystems (4 files):**

- compiler/parser.md - Parser architecture, adding syntax, error recovery
- compiler/typechecker.md - Type system internals, inference, cross-file resolution
- compiler/optimizer.md - Optimization passes, how to add new optimizations
- compiler/codegen.md - Code generation, Lua targets, source maps

**Module System (2 files):**

- module-system/resolution.md - Module resolution algorithm, circular dependencies
- module-system/type-exports.md - Cross-file type resolution, symbol transmission

**LSP Subsystem (2 files):**

- lsp/features.md - LSP feature implementation, symbol indexing
- lsp/caching.md - LSP caching framework, invalidation strategies

**Total: 14 comprehensive technical docs** (vs previous plan of only 3)

---

## Alignment with TODO.md

This documentation plan accounts for features and improvements tracked in [TODO.md](../TODO.md):

### High Priority — Enhanced Cross-File Type Support

**Documentation Impact:**

- `architecture.md` will include cross-file type resolution architecture
- `modules.md` will document type-only imports, circular type dependencies, re-exports
- `implementation.md` will guide contributors on extending type resolution

**Current State:** Type system infrastructure exists, full cross-file resolution in progress
**Documentation Status:** Will be documented based on implemented state

### Medium Priority — Incremental Parsing

**Documentation Impact:**

- `architecture.md` LSP section will cover incremental parsing architecture
  - Statement-level caching strategy
  - Dirty region calculation and offset adjustment algorithms
  - Multi-arena lifetime management approach
  - Performance characteristics and when incremental is faster/slower
- `implementation.md` will guide contributors on working with incremental parsing
  - How to work with cached parse trees
  - Implementing offset adjustment
  - Testing incremental parsing scenarios
  - Troubleshooting incremental parsing issues

**Current State:** Planned feature (TODO.md Phase 1-5)
**Documentation Status:** Will be incorporated into architecture.md and implementation.md when implemented

### Medium Priority — Better LSP Caching

**Documentation Impact:**

- `architecture.md` LSP section will cover caching architecture
  - Generic cache framework (VersionedCache, DocumentCache)
  - Cache invalidation strategies (full, partial, cascade)
  - Cross-file caching and dependency tracking
  - Memory overhead and LRU eviction
  - Performance metrics and cache effectiveness
- `implementation.md` will guide contributors on using the cache framework
  - How to add caching to new LSP features
  - Cache key design patterns
  - Cache invalidation rules
  - Testing cache correctness

**Current State:** Planned feature (TODO.md Phase 1-6)
**Documentation Status:** Will be incorporated into architecture.md and implementation.md when implemented

### Medium Priority — Language Features & Optimizer

**Documentation Impact:**

- Update `language/` pages when new features added (async/await, string patterns, etc.)
- Update `reference/configuration.md` for O2/O3 optimization levels
- Update introduction.md optimization section

**Current State:** Ongoing work
**Documentation Status:** Updated as features land

### Documentation Strategy

1. **Document current state first** — Focus on what exists now (Phase 1-3)
2. **Track implementation progress** — As TODO.md items are completed, update relevant sections in architecture.md and implementation.md
3. **Integrate, don't separate** — Incorporate new features into existing docs rather than creating separate design documents
4. **Keep docs in sync** — When code changes, update relevant documentation sections

---

## Files to Delete

| File | Reason |
| ---- | ------ |
| `docs/CLAUDE.md` | Metadata |
| `docs/LANGUAGE_FEATURES.md` | Content moves to docs-site/ |
| `docs/REFLECTION.md` | Content moves to docs-site/ |
| `docs/file-extension-migration.md` | Completed migration |
| `docs/ARENA_PERFORMANCE.md` | Folded into architecture.md |
| `docs/STRING_INTERNER.md` | Folded into architecture.md |
| `docs/BENCHMARKS.md` | Folded into architecture.md |
| `docs/profiling-guide.md` | Folded into architecture.md |
| `docs/designs/CLAUDE.md` | Metadata |
| `docs/designs/docs/designs/CLAUDE.md` | Nested dupe |
| `docs/designs/Implementation-Architecture.md` | Superseded |
| `docs/designs/Implementation-Plan.md` | Completed |
| `docs/designs/old_TODO.md` | Historical clutter |
| `docs/designs/CLI-Design.md` | Too thin, folded into language-spec |
| `docs/designs/ERROR_CODES.md` | Moves to docs-site/ reference |

## Files to Rename

| Old | New |
| --- | --- |
| `docs/ARCHITECTURE.md` | `docs/architecture.md` |
| `docs/IMPLEMENTATION.md` | `docs/implementation.md` |
| `docs/SECURITY.md` | `docs/security.md` |
| `docs/designs/LuaNext-Design.md` | `docs/designs/language-spec.md` |
| `docs/designs/AST-Structure.md` | `docs/designs/ast-structure.md` |
| `docs/designs/Grammar.md` | `docs/designs/grammar.md` |
| `docs/designs/LSP-Design.md` | `docs/designs/lsp-design.md` |
| `docs/designs/Additional-Features-Design.md` | `docs/designs/additional-features.md` |
| `docs/designs/Runtime-Validation.md` | `docs/designs/runtime-validation.md` |
