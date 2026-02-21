# Codegen Statements

Code generation for variable declarations, functions, control flow, try/catch, and multi-assignment.

## Overview

Statement codegen transforms each `Statement<'arena>` AST node into Lua code. The primary file is `crates/luanext-core/src/codegen/statements.rs`.

## Variable Declarations

```lua
const x: number = 42      → local x = 42
local y = "hello"          → local y = "hello"
global z: number = 100     → z = 100          (no local prefix)
```

### Variable Kind Mapping

| Kind | Lua Output |
| ---- | ---------- |
| `Const` | `local name = value` |
| `Local` | `local name = value` |
| `Global` | `name = value` (no `local`) |

`[NOTE: BEHAVIOR]` On Lua 5.5 targets, `Global` variables emit `global name = value` using the native `global` keyword. On all other targets, globals are bare assignments with no prefix.

### Destructuring

Array and object destructuring in variable declarations:

```lua
const [a, b] = getValues()    →  local __tmp = getValues()
                                  local a = __tmp[1]
                                  local b = __tmp[2]

const { x, y } = getPoint()  →  local __tmp = getPoint()
                                  local x = __tmp.x
                                  local y = __tmp.y
```

Global destructuring omits `local` prefix for each binding.

## Function Declarations

```lua
function greet(name: string): string {
    return "Hello " .. name
}
```

Generates:

```lua
local function greet(name)
    return "Hello " .. name
end
```

- Type annotations are erased
- Type parameters are erased
- `throws` clauses are erased
- Default parameter values generate `if param == nil then param = default end`

## Control Flow

### If / Elseif / Else

```lua
if condition then
    -- body
elseif condition2 then
    -- body
else
    -- body
end
```

### While

```lua
while condition do
    -- body
end
```

### For (Numeric)

```lua
for i = start, stop, step do
    -- body
end
```

### For (Generic / For-In)

```lua
for key, value in pairs(table) do
    -- body
end
```

With destructuring pattern, generates temporary variables and extracts fields.

### Repeat-Until

```lua
repeat
    -- body
until condition
```

### Break / Continue

- `break` → `break`
- `continue` → target-specific:
  - Lua 5.5: `continue` (native)
  - Lua 5.2-5.4: `goto __continue` + `::__continue::` label at loop end
  - Lua 5.1: not directly supported

### Return

```lua
return value1, value2   -- multiple return values
```

### Label / Goto

```lua
::label_name::
goto label_name
```

## Exception Handling

### Try-Catch-Finally

**File**: `statements.rs` — `generate_try_pcall()`

LuaNext `try/catch/finally` compiles to Lua's `pcall`:

```lua
try {
    riskyOperation()
} catch (e: Error) {
    handleError(e)
} finally {
    cleanup()
}
```

Generates:

```lua
local __ok, __result = pcall(function()
    riskyOperation()
end)
local __error = __result
if not __ok then
    local e = __error
    handleError(e)
end
-- finally
cleanup()
```

`[NOTE: BEHAVIOR]` The catch block runs inside `if not __ok then` — there was a critical bug where `else` was emitted, causing catch to run on the SUCCESS path. Fixed by removing the `else` keyword.

`[NOTE: BEHAVIOR]` `self.writeln()` (not `self.write()`) is required for the `local __error = __result` line to ensure proper newline.

### Throw

```lua
throw "Something went wrong"
```

Generates:

```lua
error("Something went wrong")
```

### Rethrow

```lua
rethrow
```

Re-throws the caught error using `error(__error)`.

### Typed Catch

```rust
enum CatchPattern<'arena> {
    Untyped { variable },
    Typed { variable, type_annotation },
    MultiTyped { variable, type_annotations },
}
```

Multiple typed catch clauses generate chained `if/elseif` checks.

## Multi-Assignment

```lua
a, b, c = expr1, expr2, expr3
```

Generates:

```lua
a, b, c = expr1, expr2, expr3
```

Direct Lua multi-assignment — no transformation needed.

## Class, Enum, Module Codegen

These have dedicated codegen files:

- Classes → [Classes](../features/classes.md), `classes.rs`
- Enums → [Enums](../features/enums.md), `enums.rs`
- Modules → [Modules](../features/modules.md), `modules.rs`
- Decorators → [Class Advanced](../features/class-advanced.md), `decorators.rs`

## Type Erasure

All type-level constructs are erased during codegen:

- Type annotations → removed
- Type parameters → removed
- Interface declarations → no output
- Type alias declarations → no output
- Type-only imports → empty (no `require()`)
- `as` type assertions → expression only (assertion removed)

## Cross-References

- [Codegen Architecture](codegen-architecture.md) — generator setup and emitter
- [Codegen Expressions](codegen-expressions.md) — expression-level codegen
- [Codegen Targets](codegen-targets.md) — target-specific differences
