---
title: Basics
---

# Basics

LuaNext extends Lua with type annotations while preserving Lua's simplicity. This guide covers variables, primitive types, and basic syntax.

## Variable Declarations

LuaNext provides three keywords for variable declarations: `const` for immutable variables, `local` for mutable block-scoped variables, and `global` for module-level variables.

### `global` — Module-Level Variables

Variables declared with `global` are accessible throughout the entire module (but not automatically exported):

```lua
global x: number = 42
global config = { debug: true }  -- Type inferred

function test(): void {
    print(x)  -- ✅ Accessible in functions
}
```

**Implicit global syntax** — Variables with type annotations at the module top-level are automatically global:

```lua
count: number = 0  -- Implicitly global (has type annotation)
x = 1              -- Assignment expression (no type annotation)
```

Compiles to:

```lua
x = 42
config = { debug = true }
count = 0

function test()
    print(x)
end
```

**Key differences from `local`:**

- Module-level scope (accessible in all functions within the module)
- Not hoisted by optimizer (already module-level)
- Not automatically exported (use `export global x = ...` to export)

### `const` — Immutable Variables

Variables declared with `const` cannot be reassigned after initialization:

```lua
const PI: number = 3.14159
const status: string = "active"

-- Error: Cannot reassign const variable
PI = 3.14  -- ❌ Type error
```

Compiles to:

```lua
local PI = 3.14159
local status = "active"
```

### `local` — Mutable Variables

Variables declared with `local` can be reassigned:

```lua
local count: number = 0
local name: string = "Alice"

count = 5        -- ✅ OK
name = "Bob"     -- ✅ OK
```

Compiles to:

```lua
local count = 0
local name = "Alice"

count = 5
name = "Bob"
```

### Why Both?

- **Type Safety** — Immutability checking catches bugs at compile time
- **Clear Intent** — Code readers know which values change
- **Better Inference** — Immutable values enable more precise type inference
- **Zero Cost** — Both compile to `local`, no runtime overhead

## Primitive Types

LuaNext includes all Lua primitive types plus additional type safety primitives.

### `nil`

Represents the absence of a value:

```lua
const x: nil = nil
local optional: string | nil = nil
```

### `boolean`

True or false values:

```lua
const isActive: boolean = true
const hasPermission: boolean = false
```

### `number`

Lua's numeric type (double-precision float):

```lua
const pi: number = 3.14159
const count: number = 42
const negative: number = -10.5
```

### `integer`

Subset of `number` for whole numbers (useful for Lua 5.3+ and array indices):

```lua
const index: integer = 1
const count: integer = 100

-- Error: Not an integer
const wrong: integer = 3.14  -- ❌ Type error
```

In Lua 5.3+, integers are stored more efficiently than floats. In earlier versions, `integer` is type-checked but compiles to `number`.

### `string`

Text values:

```lua
const name: string = "Alice"
const message: string = 'Hello, World!'
const multiline: string = [[
  This is a
  multiline string
]]
```

### `table`

Generic table type (base for all tables/objects):

```lua
const data: table = {x = 1, y = 2}
const array: table = {1, 2, 3, 4}
```

Use interfaces for typed table shapes (see [Interfaces](interfaces.md)).

### `coroutine` / `thread`

Lua coroutines:

```lua
const co: coroutine = coroutine.create(function()
    print("Hello from coroutine")
end)
```

Both `coroutine` and `thread` are synonyms for the same type.

## Special Types

### `unknown`

Type-safe unknown value. Must be narrowed before use:

```lua
const data: unknown = getExternalData()

-- Error: Cannot use unknown value directly
print(data.name)  -- ❌ Type error

-- Narrow with type guards
if type(data) == "table" then
    -- data is now table
    print(data.name)  -- ✅ OK
end
```

Unlike TypeScript's `any`, `unknown` is strict and requires type narrowing.

### `never`

Bottom type representing impossible values. Used for exhaustiveness checking:

```lua
type Status = "active" | "inactive" | "pending"

function handleStatus(status: Status): void
    if status == "active" then
        print("Active")
    elseif status == "inactive" then
        print("Inactive")
    elseif status == "pending" then
        print("Pending")
    else
        -- status is never here (exhaustive check)
        const _exhaustive: never = status
    end
end
```

### `void`

Indicates functions that return nothing:

```lua
function log(message: string): void
    print(message)
end
```

## Type Annotations

Use `:` to annotate types (same as TypeScript):

```lua
-- Variables
const name: string = "Alice"
local count: number = 0

-- Function parameters and return type
function greet(name: string): string
    return "Hello, " .. name
end

-- Inline table types
const point: {x: number, y: number} = {x = 1, y = 2}
```

## Type Inference

LuaNext infers types when not explicitly provided:

```lua
-- Inferred as number
const x = 42

-- Inferred as string
const message = "Hello"

-- Inferred as (number, number) -> number
function add(a, b)
    return a + b
end

-- Inferred as number
const sum = add(5, 10)
```

Type inference works for:

- Literals (`const x = 42` → `number`)
- Function return types (from return expressions)
- Variable assignments (from RHS type)

For complex types, explicit annotations improve readability.

## String Templates

LuaNext supports template strings with embedded expressions:

```lua
const name: string = "Alice"
const age: number = 30

const message: string = `Hello, ${name}! You are ${age} years old.`
print(message)  -- Hello, Alice! You are 30 years old.
```

Compiles to string concatenation:

```lua
local name = "Alice"
local age = 30

local message = "Hello, " .. tostring(name) .. "! You are " .. tostring(age) .. " years old."
print(message)
```

Multi-line template strings:

```lua
const user: string = "Alice"
const html: string = `
    <div>
        <h1>Welcome, ${user}!</h1>
        <p>This is a multi-line template.</p>
    </div>
`
```

## Comments

LuaNext supports Lua's comment syntax:

```lua
-- Single-line comment

--[[
  Multi-line
  comment
]]

---
--- Triple-dash comments are used for documentation
--- @param name The user's name
---
function greet(name: string): void
    print("Hello, " .. name)
end
```

## Blocks

Use `do ... end` to create scoped blocks:

```lua
do
    const x: number = 10
    print(x)  -- 10
end

-- Error: x is not in scope
print(x)  -- ❌ Not defined
```

## Type Assertions

Use `as` to assert a type:

```lua
const data: unknown = getData()
const user = data as {name: string, age: number}

print(user.name)  -- ✅ OK
```

Use with caution — type assertions bypass type checking.

## Nullable Types

Use `| nil` for nullable values:

```lua
const optional: string | nil = nil
local name: string | nil = "Alice"

name = nil  -- ✅ OK
```

Check for nil before use:

```lua
if name ~= nil then
    print(name)  -- name is string here
end
```

## Literal Types

Values can be types:

```lua
type Status = "active" | "inactive"

const status: Status = "active"  -- ✅ OK
const wrong: Status = "pending"  -- ❌ Type error
```

Number and boolean literals:

```lua
type One = 1
type Zero = 0
type True = true

const x: One = 1       -- ✅ OK
const y: Zero = 0      -- ✅ OK
const z: True = true   -- ✅ OK
```

## Arrays

Two syntaxes for array types:

```lua
-- Preferred syntax
const numbers: number[] = {1, 2, 3, 4, 5}

-- Generic syntax (equivalent)
const strings: Array<string> = {"a", "b", "c"}
```

Both compile to plain Lua tables:

```lua
local numbers = {1, 2, 3, 4, 5}
local strings = {"a", "b", "c"}
```

## Tuples

Fixed-length arrays with per-element types:

```lua
-- [type1, type2, ...]
const pair: [string, number] = {"Alice", 30}
const triple: [boolean, string, number] = {true, "test", 42}

-- Access by index
const name: string = pair[1]   -- "Alice"
const age: number = pair[2]    -- 30
```

## Next Steps

- [Control Flow](control-flow.md) — if, while, for, match
- [Functions](functions.md) — Function declarations and types
- [Type System](type-system.md) — Advanced type features
- [Interfaces](interfaces.md) — Defining table shapes

## See Also

- [Type System](type-system.md) — Union, intersection, generics
- [Standard Library](../reference/standard-library.md) — Global functions and namespaces
