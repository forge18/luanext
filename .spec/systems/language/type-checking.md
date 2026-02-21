# Type Checking

The five-phase type checker, symbol tables, type inference, narrowing, and standard library definitions.

## Overview

Type checking in LuaNext is a multi-phase process that walks the AST, resolves types, validates compatibility, and annotates expressions with inferred types. The type checker is in `crates/luanext-typechecker/`.

## Five Phases

Type checking runs in strict order per module. Each phase builds on the previous:

**File**: `crates/luanext-typechecker/src/phases/`

### Phase 1: Declaration (`declaration_phase.rs`)

Registers all top-level symbols into the symbol table without resolving their types:

- Variable declarations → `SymbolKind::Variable` or `SymbolKind::Const`
- Function declarations → `SymbolKind::Function`
- Class declarations → `SymbolKind::Class`
- Interface declarations → `SymbolKind::Interface`
- Type aliases → `SymbolKind::TypeAlias`
- Enums → `SymbolKind::Enum`
- Namespaces → `SymbolKind::Namespace`
- Parameters → `SymbolKind::Parameter`

This pass enables forward references — a function can call another function declared later in the file.

### Phase 2: Declaration Checking (`declaration_checking_phase.rs`)

Validates type-level declarations:

- Type alias cycles (no circular `type A = B; type B = A`)
- Enum member values and consistency
- Interface member types
- Class hierarchy constraints (extends/implements)

### Phase 3: Module (`module_phase.rs`)

Resolves imports and exports:

- Resolves import paths via `ModuleResolver`
- Registers imported symbols into the current scope
- Validates export declarations reference real symbols
- Handles re-exports with cycle detection
- Type-only imports validated (cannot be used at runtime)

`[NOTE: UNSAFE]` This phase uses `unsafe transmute` to convert `Symbol<'arena>` to `Symbol<'static>` via `symbol_to_static()` for cross-module symbol sharing in the `ModuleRegistry`.

### Phase 4: Inference (`inference_phase.rs`)

The main type inference pass — walks all statements and expressions:

- Infers types for variable initializers
- Checks function argument compatibility
- Resolves method calls and member access
- Performs type narrowing from guards
- Annotates `Expression.annotated_type` with inferred types

### Phase 5: Validation (`validation_phase.rs`)

Post-inference validation:

- Return type consistency
- Unused variable warnings
- Type compatibility in assignments
- Abstract class instantiation checks
- Final class/method override checks

## Symbol Table

**File**: `crates/luanext-typechecker/src/utils/symbol_table.rs`

```rust
struct Symbol<'arena> {
    name: String,
    kind: SymbolKind,
    typ: Type<'arena>,
    span: Span,
    is_exported: bool,
    references: Vec<Span>,
}

enum SymbolKind {
    Variable, Const, Function, Class,
    Interface, TypeAlias, Enum, Parameter, Namespace,
}
```

### Scope Chain

```rust
struct Scope<'arena> {
    symbols: FxHashMap<String, Symbol<'arena>>,
}

struct SymbolTable<'arena> {
    scopes: Vec<Scope<'arena>>,
}
```

Symbol lookup walks the scope chain from innermost to outermost. Each block (`if`, `while`, `for`, function body) creates a new scope.

Key operations:

| Method | Behavior |
| ------ | -------- |
| `declare(symbol)` | Add to current scope; error if duplicate |
| `lookup(name)` | Search all scopes, innermost first |
| `lookup_local(name)` | Search current scope only |
| `push_scope()` | Enter new scope |
| `pop_scope()` | Leave scope |

## Type Inference

**File**: `crates/luanext-typechecker/src/visitors/inference.rs`

The `TypeInferrer` walks expressions and returns inferred types.

### InferenceContext

```rust
struct InferenceContext<'a, 'arena> {
    access_control: &'a AccessControl,
    interner: &'a StringInterner,
    diagnostic_handler: &'a dyn DiagnosticHandler,
    class_type_params: Option<&'a [(StringId, Type<'arena>)]>,
}
```

Groups shared read-only state to stay under clippy's 7-argument limit.

### Key Inference Rules

| Expression | Inferred Type |
| ---------- | ------------- |
| `Literal::Number(f)` | `Primitive(Number)` |
| `Literal::Integer(i)` | `Primitive(Integer)` |
| `Literal::String(s)` | `Primitive(String)` |
| `Literal::Boolean(b)` | `Primitive(Boolean)` |
| `Literal::Nil` | `Primitive(Nil)` |
| `Binary(And/Or, _, _)` | `Primitive(Boolean)` |
| `Binary(Add, Number, Number)` | `Primitive(Number)` |
| `Binary(Concatenate, _, _)` | `Primitive(String)` |
| `Binary(Equal/NotEqual/LT/GT/...)` | `Primitive(Boolean)` |
| `Unary(Not, _)` | `Primitive(Boolean)` |
| `Unary(Length, _)` | `Primitive(Integer)` |
| `Unary(Negate, T)` | `T` |
| `Call(f, args)` | Return type of `f` |
| `Member(obj, prop)` | Type of `prop` in `obj` |
| `Array([...])` | `Array(union of element types)` |
| `Conditional(_, t, f)` | `Union(type(t), type(f))` |

### Method Inference

`infer_method()` resolves method calls on class instances:

1. Looks up method in class member list (`ClassMemberKind::Method`)
2. Also handles `Getter`/`Setter` access
3. Falls back to `get_`/`set_` prefix for codegen-named accessors
4. Substitutes generic type parameters via `instantiate_type()` when the class is a generic instance

`[NOTE: BEHAVIOR]` Codegen names getters/setters as `get_X()` / `set_X()` but access control registers them under original name `X`. The type checker has prefix-stripping fallback in both `infer_method()` and `infer_member()`.

## Type Narrowing

**File**: `crates/luanext-typechecker/src/visitors/narrowing.rs`

Type narrowing refines types based on control flow conditions:

| Condition | Narrowing |
| --------- | --------- |
| `if x ~= nil then` | Removes `nil` from `x`'s type |
| `if typeof(x) == "string" then` | Narrows to `string` |
| `if x instanceof MyClass then` | Narrows to `MyClass` |
| Type predicate function returns `true` | Narrows parameter to predicate type |
| Truthiness check (`if x then`) | Removes `nil` and `false` |

## Access Control

**File**: `crates/luanext-typechecker/src/visitors/access_control.rs`

Validates access modifier enforcement:

- `public` — accessible everywhere
- `private` — accessible only within the declaring class
- `protected` — accessible within the declaring class and subclasses
- `static` — accessible on the class itself, not instances

## Type Environment

**File**: `crates/luanext-typechecker/src/core/type_environment.rs`

The `TypeEnvironment` tracks:

- Type alias definitions (name → resolved type)
- Class type information
- Generic type parameter bindings

Used by `TypeCompatibility` to resolve type references.

## Standard Library

**File**: `crates/luanext-typechecker/src/state/stdlib_loader.rs`

Standard library types are loaded from `.d.luax` declaration files:

- Lua version-specific stdlib definitions
- Built-in function signatures (`print`, `require`, `pcall`, etc.)
- Global type declarations (`string`, `table`, `math`, `io`, `os`, etc.)

`[NOTE: BEHAVIOR]` Some stdlib functions (like `@sealed`, `@deprecated`) are exported as global names, which can conflict with user-defined decorator names.

## Type Formatter

**File**: `crates/luanext-typechecker/src/utils/type_formatter.rs`

Pretty-prints types for error messages and hover information.

## Type Suggestions

**File**: `crates/luanext-typechecker/src/utils/type_suggestions.rs`

Fuzzy matching for "did you mean X?" suggestions in error messages.

## Diagnostics

Type errors are reported through the `DiagnosticHandler` trait, enabling:

- CLI: Print to stderr
- LSP: Publish as editor diagnostics
- Tests: Collect in `CollectingDiagnosticHandler`

`[NOTE: BEHAVIOR]` Type errors are non-fatal — compilation continues past errors. Use `DiContainer::test()` with `CollectingDiagnosticHandler` to detect type errors in tests.

## Cross-References

- [Type Primitives](type-primitives.md) — type definitions
- [Type Advanced](type-advanced.md) — generics, conditional types
- [Type Compatibility](type-compatibility.md) — assignability rules
- [Module Resolution](../features/module-resolution.md) — module phase details
- [Classes](../features/classes.md) — class type checking
- [Incremental Cache](../compiler/incremental-cache.md) — caching type-checked results
