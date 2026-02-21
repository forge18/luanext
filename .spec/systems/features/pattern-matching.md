# Pattern Matching

Match expressions, pattern kinds, guards, and destructuring.

## Overview

LuaNext supports pattern matching via `match` expressions with multiple arms, guards, and destructuring patterns. Patterns are also used in variable declarations and for-in loops.

**Parser**: `crates/luanext-parser/src/ast/pattern.rs`, `crates/luanext-parser/src/ast/expression.rs` (`MatchExpression`)
**Codegen**: `crates/luanext-core/src/codegen/patterns.rs`

## Match Expressions

```rust
struct MatchExpression<'arena> {
    value: &'arena Expression<'arena>,
    arms: &'arena [MatchArm<'arena>],
    span: Span,
}

struct MatchArm<'arena> {
    pattern: Pattern<'arena>,
    guard: Option<Expression<'arena>>,
    body: MatchArmBody<'arena>,
    span: Span,
}

enum MatchArmBody<'arena> {
    Expression(&'arena Expression<'arena>),
    Block(Block<'arena>),
}
```

### Syntax

```lua
const result = match value {
    when 1 => "one"
    when 2 | 3 => "two or three"
    when x when x > 10 => "big: " .. tostring(x)
    when _ => "other"
}
```

Arms use the `when` keyword. Each arm has a pattern, an optional guard, and a body (expression or block).

## Pattern Kinds

```rust
enum Pattern<'arena> {
    Identifier(Ident),
    Literal(Literal, Span),
    Array(ArrayPattern<'arena>),
    Object(ObjectPattern<'arena>),
    Wildcard(Span),
    Or(OrPattern<'arena>),
    Template(TemplatePattern<'arena>),
}
```

### Identifier Pattern

Binds the matched value to a name:

```lua
when x => x * 2
```

### Literal Pattern

Matches a specific constant value:

```lua
when 42 => "the answer"
when "hello" => "greeting"
when true => "yes"
when nil => "nothing"
```

### Wildcard Pattern

Matches anything, binds nothing:

```lua
when _ => "default"
```

### Or Pattern

Matches any of several alternatives:

```lua
when 1 | 2 | 3 => "small number"
```

```rust
struct OrPattern<'arena> {
    alternatives: &'arena [Pattern<'arena>],
    span: Span,
}
```

### Array Pattern

Destructures arrays/tables with sequential integer keys:

```lua
when [first, second, ...rest] => first + second
when [_, _, third] => third
when [head, ...] => head
```

```rust
struct ArrayPattern<'arena> {
    elements: &'arena [ArrayPatternElement<'arena>],
    span: Span,
}

enum ArrayPatternElement<'arena> {
    Pattern(PatternWithDefault<'arena>),   // element with optional default
    Rest(Ident),                           // ...rest
    Hole,                                  // skip position
}

struct PatternWithDefault<'arena> {
    pattern: Pattern<'arena>,
    default: Option<Expression<'arena>>,
}
```

### Object Pattern

Destructures table properties by key:

```lua
when { name, age } => name .. " is " .. tostring(age)
when { x: posX, y: posY } => posX + posY
when { type: "error", message, ...rest } => message
```

```rust
struct ObjectPattern<'arena> {
    properties: &'arena [ObjectPatternProperty<'arena>],
    rest: Option<Ident>,
    span: Span,
}

struct ObjectPatternProperty<'arena> {
    key: Ident,
    computed_key: Option<Expression<'arena>>,
    value: Option<Pattern<'arena>>,     // rename binding
    default: Option<Expression<'arena>>,
    span: Span,
}
```

### Template Pattern

Matches strings against templates with captures:

```lua
when `hello ${name}` => "Hi " .. name
when `${prefix}-${suffix}` => prefix .. "/" .. suffix
```

```rust
struct TemplatePattern<'arena> {
    parts: &'arena [TemplatePatternPart],
    span: Span,
}

enum TemplatePatternPart {
    String(String),     // literal segment
    Capture(Ident),     // captured variable
}
```

## Guards

Guards add conditions to match arms:

```lua
when x when x > 0 and x < 100 => "in range"
when [a, b] when a == b => "equal pair"
```

The guard expression is evaluated after the pattern matches. If the guard is `false`, the arm is skipped and the next arm is tried.

## Destructuring in Declarations

Patterns are also used in variable declarations:

```lua
const [first, second, ...rest] = getItems()
const { name, age } = getPerson()
const [x, y] = getCoords()
```

## Destructuring in For-In

For-in loops support destructuring:

```lua
for [key, value] in pairs(table) do
    print(key, value)
end

for { x, y } in points do
    print(x, y)
end
```

`[NOTE: BEHAVIOR]` For-in destructuring requires typed arrays (e.g., `number[][]` or `{x: number}[]`) for the type checker to validate element types.

## Codegen

Match expressions compile to a series of `if/elseif` chains:

```lua
-- match x { when 1 => "one", when 2 => "two", when _ => "other" }
local __match_result
if x == 1 then
    __match_result = "one"
elseif x == 2 then
    __match_result = "two"
else
    __match_result = "other"
end
```

Array destructuring generates indexed access, object destructuring generates field access.

## Cross-References

- [Enums](enums.md) — enum values in match patterns
- [AST](../language/ast.md) — Pattern, MatchExpression nodes
- [Codegen Expressions](../compiler/codegen-expressions.md) — match expression codegen
