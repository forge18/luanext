# Codegen Expressions

Code generation for operators, calls, optional chaining, pipe, ternary, and templates.

## Overview

Expression codegen transforms `ExpressionKind<'arena>` nodes into Lua expressions. Split across multiple files in `crates/luanext-core/src/codegen/expressions/`.

**Files**: `expressions.rs`, `expressions/binary_ops.rs`, `expressions/calls.rs`, `expressions/literals.rs`

## Literals

| AST Literal | Lua Output |
| ----------- | ---------- |
| `Nil` | `nil` |
| `Boolean(true)` | `true` |
| `Boolean(false)` | `false` |
| `Number(f)` | `f` (numeric literal) |
| `Integer(i)` | `i` (integer literal) |
| `String(s)` | `"s"` (quoted string) |

## Identifiers

`Identifier(StringId)` → resolved via `StringInterner` to the variable name.

`SelfKeyword` → `self`
`SuperKeyword` → depends on context (parent class reference)

## Binary Operators

| BinaryOp | Lua Output | Notes |
| -------- | ---------- | ----- |
| `Add` | `a + b` | |
| `Subtract` | `a - b` | |
| `Multiply` | `a * b` | |
| `Divide` | `a / b` | |
| `Modulo` | `a % b` | |
| `Power` | `a ^ b` | |
| `IntegerDivide` | target-specific | `//` on 5.3+, `math.floor(a/b)` on 5.1/5.2 |
| `Equal` | `a == b` | |
| `NotEqual` | `a ~= b` | `!=` in source → `~=` in Lua |
| `LessThan` | `a < b` | |
| `LessThanOrEqual` | `a <= b` | |
| `GreaterThan` | `a > b` | |
| `GreaterThanOrEqual` | `a >= b` | |
| `And` | `a and b` | |
| `Or` | `a or b` | |
| `NullCoalesce` | special | See below |
| `Concatenate` | `a .. b` | |
| `BitwiseAnd` | target-specific | `&` on 5.3+, helper on 5.1/5.2 |
| `BitwiseOr` | target-specific | |
| `BitwiseXor` | target-specific | |
| `ShiftLeft` | target-specific | |
| `ShiftRight` | target-specific | |
| `Instanceof` | `__instanceof(a, b)` | Runtime check |

### Null Coalescing (`??`)

```lua
a ?? b
```

Generates:

```lua
(function() local __t = a; if __t ~= nil then return __t else return b end end)()
```

Or simpler form if `a` has no side effects: uses `a or b` when `a` cannot be `false`.

## Unary Operators

| UnaryOp | Lua Output |
| ------- | ---------- |
| `Not` | `not a` |
| `Negate` | `-a` |
| `Length` | `#a` |
| `BitwiseNot` | target-specific (`~a` on 5.3+) |

## Assignment Operators

Compound assignments desugar to binary operations:

```lua
x += 5    →    x = x + 5
x ..= s   →    x = x .. s
x //= 2   →    x = x // 2  (or math.floor on older targets)
```

## Member Access

```lua
obj.prop     →    obj.prop
obj["key"]   →    obj["key"]
```

## Method Calls

```lua
obj::method(args)    →    obj:method(args)
```

`[NOTE: BEHAVIOR]` LuaNext uses `::` for method calls (double colon), which compiles to Lua's `:` (single colon). The `::` syntax avoids ambiguity with Lua's label syntax `::name::`.

## Function Calls

```lua
func(arg1, arg2)    →    func(arg1, arg2)
```

With type arguments (erased):

```lua
func<T>(arg1)       →    func(arg1)
```

With spread:

```lua
func(...args)       →    func(unpack(args))   -- or table.unpack on 5.2+
```

## Optional Chaining

### Optional Member (`?.`)

```lua
obj?.prop
```

Generates:

```lua
(obj ~= nil) and obj.prop or nil
```

### Optional Index (`?[]`)

```lua
arr?[0]
```

### Optional Call (`?.()`)

```lua
fn?.(args)
```

### Optional Method Call (`?.::`)

```lua
obj?.::method(args)
```

All generate nil-guarded access patterns.

## Pipe Operator (`|>`)

```lua
value |> transform |> format
```

Generates:

```lua
format(transform(value))
```

The pipe operator is purely syntactic — it nests function calls.

## Ternary / Conditional

```lua
condition ? trueExpr : falseExpr
```

Generates:

```lua
(function() if condition then return trueExpr else return falseExpr end end)()
```

Or uses `condition and trueExpr or falseExpr` when safe (trueExpr is not falsy).

## Match Expressions

See [Pattern Matching](../features/pattern-matching.md) for codegen details.

## Template Literals

```lua
`hello ${name}, you are ${age} years old`
```

Generates:

```lua
"hello " .. tostring(name) .. ", you are " .. tostring(age) .. " years old"
```

Parts are concatenated with `..`, interpolated expressions wrapped in `tostring()`.

## New (Constructor)

```lua
new MyClass(arg1, arg2)
```

Generates:

```lua
MyClass.new(arg1, arg2)
```

## Type Assertions

```lua
expr as Type
```

Generates just `expr` — the assertion is erased.

## Array Literals

```lua
[1, 2, 3]
```

Generates:

```lua
{1, 2, 3}
```

With spread:

```lua
[...arr, 4, 5]
```

Generates table construction with `table.move` or loop-based spread.

## Object Literals

```lua
{ name: "Alice", age: 30 }
```

Generates:

```lua
{ name = "Alice", age = 30 }
```

With spread:

```lua
{ ...base, extra: true }
```

Generates table merge code.

## Arrow Functions

```lua
(x, y) => x + y
```

Generates:

```lua
function(x, y) return x + y end
```

Block body arrows:

```lua
(x) => { return x * 2 }    →    function(x) return x * 2 end
```

## Try Expressions

```lua
expr try catch(e) fallback
```

Generates a `pcall` wrapper returning the result or fallback.

## Error Chain

```lua
expr ?? handler
```

Similar to null coalescing but for error propagation.

## Cross-References

- [Codegen Architecture](codegen-architecture.md) — emitter and generator setup
- [Codegen Statements](codegen-statements.md) — statement-level codegen
- [Codegen Targets](codegen-targets.md) — target-specific operator codegen
- [Pattern Matching](../features/pattern-matching.md) — match expression codegen
