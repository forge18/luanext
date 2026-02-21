# Type Advanced

Generics, conditional types, mapped types, and other advanced type-level constructs.

## Overview

LuaNext's type system includes TypeScript-inspired advanced types for expressing complex relationships. These are primarily resolved during the type-checking phases in `luanext-typechecker`.

**AST definitions**: `crates/luanext-parser/src/ast/types.rs`
**Resolution logic**: `crates/luanext-typechecker/src/types/`

## Generics

### Type Parameters

```rust
struct TypeParameter<'arena> {
    name: Ident,
    constraint: Option<&'arena Type<'arena>>,   // extends clause
    default: Option<&'arena Type<'arena>>,       // default type
    span: Span,
}
```

Type parameters appear on functions, classes, interfaces, type aliases, and methods:

```lua
function identity<T>(x: T): T { return x }
class Container<T extends Serializable> { ... }
type Pair<A, B = A> = { first: A, second: B }
```

### Type Arguments

Type arguments are provided at usage sites via `TypeReference.type_arguments`:

```lua
const c: Container<string> = new Container<string>()
```

### Generic Instantiation

**File**: `crates/luanext-typechecker/src/types/generics.rs`

Generics are instantiated by substituting type parameters with concrete type arguments. The type checker performs this substitution when:

1. A generic function is called with explicit or inferred type arguments
2. A generic class/interface is used with type arguments
3. A generic type alias is expanded

`[NOTE: PARTIAL]` Cross-module generic type argument handling is implemented via `apply_type_arguments()` helper but full generic instantiation across module boundaries is limited to basic cases.

## Conditional Types

```rust
TypeKind::Conditional(ConditionalType<'arena>)

struct ConditionalType<'arena> {
    check_type: &'arena Type<'arena>,
    extends_type: &'arena Type<'arena>,
    true_type: &'arena Type<'arena>,
    false_type: &'arena Type<'arena>,
    span: Span,
}
```

`T extends U ? A : B` — evaluates to `A` if `T` is assignable to `U`, otherwise `B`.

```lua
type IsString<T> = T extends string ? true : false
```

### Distribution

When the check type is a bare type parameter and the input is a union, the conditional distributes over each member:

```lua
type ToArray<T> = T extends unknown ? T[] : never
// ToArray<string | number> = string[] | number[]
```

`[NOTE: BEHAVIOR]` Distribution only occurs when the check type is a naked type parameter, matching TypeScript semantics.

## Mapped Types

```rust
TypeKind::Mapped(MappedType<'arena>)

struct MappedType<'arena> {
    readonly_modifier: MappedTypeModifier,
    type_parameter: &'arena TypeParameter<'arena>,
    in_type: &'arena Type<'arena>,
    optional_modifier: MappedTypeModifier,
    value_type: &'arena Type<'arena>,
    span: Span,
}

enum MappedTypeModifier {
    Add,      // +readonly, +?
    Remove,   // -readonly, -?
    None,     // no modifier change
}
```

Transform each property of a type: `{ [K in keyof T]: Wrapper<T[K]> }`

### Modifier Manipulation

- `+readonly` / `-readonly` — add or remove readonly from properties
- `+?` / `-?` — add or remove optionality

```lua
type ReadonlyAll<T> = { +readonly [K in keyof T]: T[K] }
type Required<T> = { [K in keyof T]-?: T[K] }
```

## KeyOf

```rust
TypeKind::KeyOf(&'arena Type<'arena>)
```

`keyof T` — produces a union of all property name literal types.

```lua
interface Point { x: number, y: number }
type PointKeys = keyof Point   // "x" | "y"
```

## Index Access Types

```rust
TypeKind::IndexAccess(&'arena Type<'arena>, &'arena Type<'arena>)
```

`T[K]` — lookup a property type by key.

```lua
type Name = Person["name"]   // string
```

## Template Literal Types

```rust
TypeKind::TemplateLiteral(TemplateLiteralType<'arena>)

struct TemplateLiteralType<'arena> {
    parts: &'arena [TemplateLiteralTypePart<'arena>],
    span: Span,
}

enum TemplateLiteralTypePart<'arena> {
    String(String),
    Type(Type<'arena>),
}
```

Construct string literal types from parts:

```lua
type Greeting<N extends string> = `hello ${N}`
// Greeting<"world"> = "hello world"
```

## Infer

```rust
TypeKind::Infer(Ident)
```

`infer R` — declares a type variable to be inferred within a conditional type's `extends` clause:

```lua
type ReturnType<T> = T extends (...args: unknown[]) -> infer R ? R : never
```

The `Ident` names the inferred variable (e.g., `R`), which is then available in the true branch.

## Type Predicates

```rust
TypeKind::TypePredicate(TypePredicate<'arena>)

struct TypePredicate<'arena> {
    parameter_name: Ident,
    type_annotation: &'arena Type<'arena>,
    span: Span,
}
```

`param is Type` — used as return types for type guard functions:

```lua
function isString(x: unknown): x is string {
    return typeof(x) == "string"
}
```

When the function returns `true`, the type checker narrows the parameter's type in the calling scope.

## Variadic Types

```rust
TypeKind::Variadic(&'arena Type<'arena>)
```

`...T[]` — represents a variable-length list of a type, used in function return types and tuple rest elements.

```lua
function multi(): ...number[] { return 1, 2, 3 }
```

## Utility Types

**File**: `crates/luanext-typechecker/src/types/utility_types.rs`

Built-in utility types implemented via the type-checking machinery:

| Utility | Definition | Purpose |
| ------- | ---------- | ------- |
| `Partial<T>` | `{ [K in keyof T]?: T[K] }` | All properties optional |
| `Required<T>` | `{ [K in keyof T]-?: T[K] }` | All properties required |
| `Readonly<T>` | `{ +readonly [K in keyof T]: T[K] }` | All properties readonly |
| `Pick<T, K>` | Selected properties | Subset of properties |
| `Omit<T, K>` | Excluded properties | Complement of Pick |
| `Record<K, V>` | `{ [key: K]: V }` | Dictionary type |
| `Exclude<T, U>` | Conditional filter | Remove types from union |
| `Extract<T, U>` | Conditional filter | Keep types in union |
| `NonNullable<T>` | `Exclude<T, nil>` | Remove nil from union |
| `ReturnType<T>` | `T extends (...) -> infer R ? R : never` | Extract return type |
| `Parameters<T>` | Conditional infer | Extract parameter tuple |

`[NOTE: PARTIAL]` Not all utility types may be fully functional across all usage contexts. Basic usage is well-tested; complex compositions may have gaps.

## Cross-References

- [Type Primitives](type-primitives.md) — base types these operate on
- [Type Compatibility](type-compatibility.md) — how advanced types interact with assignability
- [Type Checking](type-checking.md) — phases that resolve these types
- [Classes](../features/classes.md) — generic classes
- [Modules](../features/modules.md) — generic imports/exports
