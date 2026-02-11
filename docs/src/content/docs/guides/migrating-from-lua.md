---
title: Migrating from Lua
---

# Migrating from Lua

This guide helps you migrate existing Lua code to LuaNext, taking advantage of static typing and modern features while maintaining compatibility.

## Overview

LuaNext is a **gradual typing** system, meaning you can:

1. Start with zero type annotations (valid Lua is valid LuaNext)
2. Add types incrementally where they provide value
3. Gradually enable modern features (OOP, FP, decorators)
4. Maintain full compatibility with existing Lua code

## Migration Strategy

### Quick Start: Three-Step Migration

1. **Rename files** — Change `.lua` to `.luax`
2. **Compile** — Run `luanext your-file.luax` (should work immediately)
3. **Add types gradually** — Start with function signatures, then variables

### Recommended Approach

For large codebases:

1. **Start at the edges** — Add types to public APIs first
2. **Work inward** — Type internal modules once APIs are typed
3. **Use `unknown`** — For complex external data, use `unknown` and narrow
4. **Enable features incrementally** — Turn on `strictNullChecks`, OOP, FP as needed

## Basic Syntax Changes

### Variable Declarations

**Lua:**
```lua
local x = 10
local name = "Alice"
```

**LuaNext:**
```lua
-- Mutable variables (same as Lua)
local x: number = 10
local name: string = "Alice"

-- Immutable variables (new)
const PI: number = 3.14159
const status: string = "active"
```

**Migration tip:** Use `const` for values that never change. This catches accidental reassignment bugs and enables better type inference.

### Function Declarations

**Lua:**
```lua
function add(a, b)
    return a + b
end

function greet(name)
    print("Hello, " .. name)
end
```

**LuaNext:**
```lua
-- Add parameter and return types
function add(a: number, b: number): number
    return a + b
end

function greet(name: string): void
    print("Hello, " .. name)
end

-- Types can be inferred for simple cases
function double(x: number)  -- return type inferred as number
    return x * 2
end
```

**Migration tip:** Start by adding types to function signatures. This documents intent and catches type errors at call sites.

### Table Types

**Lua:**
```lua
local point = {x = 10, y = 20}

function distance(p1, p2)
    local dx = p2.x - p1.x
    local dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
end
```

**LuaNext:**
```lua
-- Define table shape with interface
interface Point
    x: number
    y: number
end

const point: Point = {x = 10, y = 20}

function distance(p1: Point, p2: Point): number
    const dx = p2.x - p1.x
    const dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
end

-- Or use inline types for one-offs
const config: {host: string, port: number} = {
    host = "localhost",
    port = 8080
}
```

**Migration tip:** Create interfaces for tables used in multiple places. Use inline types for local, one-off tables.

### Arrays

**Lua:**
```lua
local numbers = {1, 2, 3, 4, 5}
local names = {"Alice", "Bob", "Charlie"}
```

**LuaNext:**
```lua
-- Add array type annotations
const numbers: number[] = {1, 2, 3, 4, 5}
const names: string[] = {"Alice", "Bob", "Charlie"}

-- Generic syntax (equivalent)
const items: Array<string> = {"a", "b", "c"}
```

## Handling Nil

### Nullable Types

**Lua:**
```lua
local value = nil
value = "hello"

function findUser(id)
    -- May return nil
    return users[id]
end
```

**LuaNext:**
```lua
-- Explicitly allow nil with union type
local value: string | nil = nil
value = "hello"

function findUser(id: string): User | nil
    return users[id]
end

-- Check for nil before use
const user = findUser("123")
if user ~= nil then
    print(user.name)  -- user is User here (narrowed)
end
```

**Migration tip:** Enable `strictNullChecks` in config to catch nil-related bugs. This forces explicit nil handling.

### Optional Chaining (New Feature)

**Lua:**
```lua
-- Verbose nil checking
local city = nil
if user ~= nil and user.address ~= nil then
    city = user.address.city
end
```

**LuaNext:**
```lua
-- Optional chaining
const city = user?.address?.city  -- nil if any part is nil

-- Optional method calls
const result = obj?:method()  -- Only calls if obj is not nil
```

## Replacing Metatables with Classes

### Lua Metatable Pattern

**Lua:**
```lua
local Vector = {}
Vector.__index = Vector

function Vector.new(x, y)
    local self = setmetatable({}, Vector)
    self.x = x
    self.y = y
    return self
end

function Vector:add(other)
    return Vector.new(self.x + other.x, self.y + other.y)
end

function Vector:magnitude()
    return math.sqrt(self.x * self.x + self.y * self.y)
end

local v1 = Vector.new(1, 2)
local v2 = Vector.new(3, 4)
local v3 = v1:add(v2)
```

**LuaNext (with OOP enabled):**
```lua
-- Class syntax replaces metatable boilerplate
class Vector(public x: number, public y: number)
    function add(other: Vector): Vector
        return Vector.new(self.x + other.x, self.y + other.y)
    end

    function magnitude(): number
        return math.sqrt(self.x * self.x + self.y * self.y)
    end
end

const v1 = Vector.new(1, 2)
const v2 = Vector.new(3, 4)
const v3 = v1:add(v2)
```

**Migration tip:** Classes compile to metatables, so the generated code is similar to hand-written Lua. Class features are available by default.

### Operator Overloading

**Lua:**
```lua
function Vector:__add(other)
    return Vector.new(self.x + other.x, self.y + other.y)
end

Vector.__mul = function(self, scalar)
    return Vector.new(self.x * scalar, self.y * scalar)
end
```

**LuaNext:**
```lua
class Vector(public x: number, public y: number)
    operator +(other: Vector): Vector
        return Vector.new(self.x + other.x, self.y + other.y)
    end

    operator *(scalar: number): Vector
        return Vector.new(self.x * scalar, self.y * scalar)
    end
end

const v1 = Vector.new(1, 2)
const v2 = Vector.new(3, 4)
const v3 = v1 + v2       -- Uses operator +
const v4 = v1 * 2        -- Uses operator *
```

## String Operations

### String Concatenation

**Lua:**
```lua
local message = "Hello, " .. name .. "! You are " .. age .. " years old."
```

**LuaNext:**
```lua
-- Template strings (more readable)
const message = `Hello, ${name}! You are ${age} years old.`

-- Compiles to concatenation, same as Lua
-- local message = "Hello, " .. tostring(name) .. "! You are " .. tostring(age) .. " years old."
```

## Error Handling

### pcall/xpcall Pattern

**Lua:**
```lua
local success, result = pcall(function()
    return riskyOperation()
end)

if not success then
    print("Error: " .. result)
else
    print("Result: " .. result)
end
```

**LuaNext:**
```lua
-- Try/catch syntax (compiles to pcall)
try
    const result = riskyOperation()
    print("Result: " .. result)
catch error
    print("Error: " .. error)
end

-- Try expression for simple fallbacks
const result = try riskyOperation() catch _ => defaultValue

-- Error chain operator (shorthand)
const result = riskyOperation() !! defaultValue
```

## Module System

### Lua Modules

**Lua:**
```lua
-- math-utils.lua
local M = {}

function M.add(a, b)
    return a + b
end

function M.multiply(a, b)
    return a * b
end

return M

-- main.lua
local utils = require("math-utils")
print(utils.add(5, 3))
```

**LuaNext:**
```lua
-- math-utils.luax
export function add(a: number, b: number): number
    return a + b
end

export function multiply(a: number, b: number): number
    return a * b
end

-- main.luax
import {add, multiply} from "./math-utils"
print(add(5, 3))

-- Or import everything
import * as utils from "./math-utils"
print(utils.add(5, 3))
```

**Migration tip:** The ES6-style module system compiles to Lua's `require` and module pattern. It provides better tooling support (autocomplete, refactoring).

## Control Flow Enhancements

### Pattern Matching

**Lua:**
```lua
function getStatusMessage(code)
    if code == 200 then
        return "OK"
    elseif code == 404 then
        return "Not Found"
    elseif code == 500 then
        return "Internal Server Error"
    else
        return "Unknown Status"
    end
end
```

**LuaNext (with FP enabled):**
```lua
function getStatusMessage(code: number): string
    return match code
        | 200 -> "OK"
        | 404 -> "Not Found"
        | 500 -> "Internal Server Error"
        | _ -> "Unknown Status"
    end
end
```

**Migration tip:** Pattern matching and functional features are available by default. Pattern matching compiles to if-else chains, no runtime overhead.

## Type Annotations for Existing APIs

### Standard Library

**Lua:**
```lua
local result = table.concat(items, ", ")
local index = string.find(text, "pattern")
```

**LuaNext:**
```lua
-- LuaNext provides types for standard library
const result: string = table.concat(items, ", ")
const index: number | nil = string.find(text, "pattern")

-- Type checker knows standard library signatures
table.insert(numbers, "wrong")  -- ❌ Type error
```

### External Libraries

For libraries without types:

```lua
-- Declare types for external library
declare module "external-lib"
    export function process(data: string): number
    export interface Config
        timeout: number
        retries: number
    end
end

import {process, type Config} from "external-lib"

const result: number = process("data")
const config: Config = {timeout = 1000, retries = 3}
```

## Configuration for Migration

### Gradual Adoption Config

Start with minimal type checking:

```yaml
# luanext.config.yaml
compilerOptions:
  target: "5.1"              # Match your Lua version
  strictNullChecks: false    # Allow nil anywhere (Lua default)
  enableDecorators: false    # Keep it simple
  outDir: "dist"
```

### Progressive Strictness

As you add types, increase strictness:

```yaml
compilerOptions:
  target: "5.4"
  strictNullChecks: true     # Enable once nil handling is typed
  enableDecorators: true     # Enable for metadata features
  outDir: "dist"
```

## Common Patterns

### Iterators

**Lua:**
```lua
for i, value in ipairs(array) do
    print(i, value)
end

for key, value in pairs(table) do
    print(key, value)
end
```

**LuaNext (same syntax, with types):**
```lua
const array: number[] = {1, 2, 3, 4, 5}

for i, value in ipairs(array) do
    -- i is number, value is number
    print(i, value)
end

interface Data
    [key: string]: any
end

const data: Data = {x = 1, y = 2}

for key, value in pairs(data) do
    -- key is string, value is any
    print(key, value)
end
```

### Varargs

**Lua:**
```lua
function sum(...)
    local total = 0
    for _, n in ipairs({...}) do
        total = total + n
    end
    return total
end
```

**LuaNext:**
```lua
function sum(...numbers: number[]): number
    local total = 0
    for _, n in ipairs(numbers) do
        total = total + n
    end
    return total
end

print(sum(1, 2, 3, 4, 5))  -- 15
```

### Callbacks

**Lua:**
```lua
function processItems(items, callback)
    for _, item in ipairs(items) do
        callback(item)
    end
end

processItems(numbers, function(n)
    print(n * 2)
end)
```

**LuaNext:**
```lua
function processItems(items: number[], callback: (n: number) => void): void
    for _, item in ipairs(items) do
        callback(item)
    end
end

processItems(numbers, (n) => {
    print(n * 2)
})

-- Or with traditional syntax
processItems(numbers, function(n: number): void
    print(n * 2)
end)
```

## Compatibility Considerations

### Lua Version Targeting

LuaNext compiles to different Lua versions:

```yaml
compilerOptions:
  target: "5.1"  # Uses compatibility patterns
  # OR
  target: "5.3"  # Uses integer type, bitwise operators
  # OR
  target: "5.4"  # Uses latest features
```

**5.1 → 5.4 differences:**
- **Integers** — `integer` type is type-checked in 5.1/5.2, native in 5.3+
- **Bitwise operators** — Compile to bit library calls in 5.1/5.2, native in 5.3+
- **`goto`** — Available in 5.2+
- **Integer division (`//`)** — Emulated in 5.1/5.2, native in 5.3+

### C API and FFI

LuaNext is **source compatible** only. For C extensions:

```lua
-- Declare C module types
declare module "cjson"
    export function encode(data: any): string
    export function decode(json: string): any
end

import * as cjson from "cjson"

const json: string = cjson.encode({x = 1, y = 2})
```

### Metatables Still Work

LuaNext classes compile to metatables, so existing metatable code is compatible:

```lua
class MyClass
    -- LuaNext class
end

-- Still accessible as metatable in generated Lua
local mt = getmetatable(MyClass.new())
print(mt.__index)  -- Works as expected
```

## Migration Checklist

### Phase 1: Setup
- [ ] Install LuaNext compiler
- [ ] Rename `.lua` files to `.luax`
- [ ] Create `luanext.config.yaml` with minimal settings
- [ ] Verify project compiles without errors

### Phase 2: Add Types
- [ ] Add types to public API functions
- [ ] Add types to complex data structures (interfaces)
- [ ] Add types to frequently-used utility functions
- [ ] Enable `strictNullChecks` when ready

### Phase 3: Modern Features
- [ ] Convert metatables to classes
- [ ] Add pattern matching for complex conditionals
- [ ] Use decorators for metadata (enable with `enableDecorators: true`)

### Phase 4: Refinement
- [ ] Add types to remaining untyped code
- [ ] Enable all strict checks
- [ ] Review and fix type errors
- [ ] Document types for external consumers

## Troubleshooting

### "Type mismatch" errors

**Problem:** Lua allows mixing types, but LuaNext doesn't.

**Solution:** Use union types or type assertions:
```lua
-- Union type
local value: string | number = getValue()

-- Type assertion (use sparingly)
const data = externalCall() as MyType
```

### "Cannot assign to const"

**Problem:** Variable declared with `const` cannot be reassigned.

**Solution:** Use `local` for mutable variables:
```lua
local count = 0
count = count + 1  -- OK
```

### "Missing nil check"

**Problem:** `strictNullChecks` requires explicit nil handling.

**Solution:** Check for nil or use optional chaining:
```lua
-- Explicit nil check
if value ~= nil then
    print(value.property)
end

-- Optional chaining
print(value?.property)
```

### "Module not found"

**Problem:** Import path doesn't match file structure.

**Solution:** Use relative paths:
```lua
-- From src/main.luax importing src/utils.luax
import {helper} from "./utils"

-- From src/main.luax importing src/lib/utils.luax
import {helper} from "./lib/utils"
```

## Performance Considerations

### Zero Runtime Cost

All LuaNext features compile to plain Lua with **no runtime overhead**:

- **Types** — Erased at compile time
- **Interfaces** — Type-checking only, no runtime representation
- **Classes** — Compile to metatables (same as hand-written Lua)
- **Pattern matching** — Compiles to if-else chains
- **Template strings** — Compile to concatenation

### Same Performance as Lua

```lua
-- LuaNext
const result: number = add(5, 3)

-- Compiles to (same as hand-written Lua)
local result = add(5, 3)
```

## Next Steps

- [Language Basics](../language/basics.md) — Core syntax and types
- [Type System](../language/type-system.md) — Advanced type features
- [Classes](../language/classes.md) — OOP with classes
- [Pattern Matching](../language/pattern-matching.md) — Functional patterns
- [Configuration](../reference/configuration.md) — Compiler options

## See Also

- [Lua Targets](lua-targets.md) — Targeting different Lua versions
- [Standard Library](../reference/standard-library.md) — Typed Lua standard library
- [CLI Reference](../reference/cli.md) — Command-line options
