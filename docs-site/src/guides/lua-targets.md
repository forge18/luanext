# Targeting Different Lua Versions

LuaNext can target Lua versions 5.1, 5.2, 5.3, and 5.4. This guide explains the differences, compatibility patterns, and how to choose the right target for your project.

## Quick Reference

| Version | Released | Key Features | Compatibility |
|---------|----------|--------------|---------------|
| **Lua 5.1** | 2006 | Module system, `setfenv`/`getfenv` | Widest compatibility |
| **Lua 5.2** | 2011 | `goto`, `_ENV`, bit32 library | Breaking changes from 5.1 |
| **Lua 5.3** | 2015 | Native integers, bitwise operators | Mostly compatible with 5.2 |
| **Lua 5.4** | 2020 | `<const>`, `<close>`, new metamethods | Latest features |

## Setting the Target

### In Configuration File

```yaml
# luanext.config.yaml
compilerOptions:
  target: "5.4"  # Options: "5.1", "5.2", "5.3", "5.4"
```

### Command Line

```bash
# Target Lua 5.1
luanext --target 5.1 main.luax

# Target Lua 5.4
luanext --target 5.4 main.luax
```

### Default Target

If not specified, LuaNext targets **Lua 5.4** (latest features).

## Choosing a Target

### Use Lua 5.1 When

- **Maximum compatibility needed** — Most embedded Lua uses 5.1
- **Legacy systems** — Older games, applications using Lua 5.1
- **Widest deployment** — 5.1 code runs everywhere
- **Embedded environments** — Many game engines and applications use 5.1

### Use Lua 5.2 When

- **You need `goto`** — For complex control flow (state machines, etc.)
- **`_ENV` manipulation needed** — For sandboxing or environment control
- **5.1 is too old** — But 5.3+ features aren't critical

### Use Lua 5.3 When

- **Native integers needed** — For performance with whole numbers
- **Bitwise operators** — For bit manipulation (games, protocols)
- **UTF-8 library** — For Unicode string handling
- **Better performance** — Integer optimizations help

### Use Lua 5.4 When

- **Latest features** — `<const>`, `<close>`, generational GC
- **New metamethods** — `__close` for resource management
- **Best performance** — Latest optimizations
- **No legacy constraints** — New projects with modern Lua

## Feature Compatibility Matrix

### Language Features

| Feature | 5.1 | 5.2 | 5.3 | 5.4 | Notes |
|---------|-----|-----|-----|-----|-------|
| Basic syntax | ✅ | ✅ | ✅ | ✅ | — |
| Metatables | ✅ | ✅ | ✅ | ✅ | — |
| Coroutines | ✅ | ✅ | ✅ | ✅ | — |
| `goto` | ❌ | ✅ | ✅ | ✅ | Emulated in 5.1 |
| `_ENV` | ❌ | ✅ | ✅ | ✅ | Uses `setfenv` in 5.1 |
| Native integers | ❌ | ❌ | ✅ | ✅ | Type-checked only in 5.1/5.2 |
| Bitwise ops | ❌ | ❌ | ✅ | ✅ | Uses bit32/bit library in 5.1/5.2 |
| Integer division (`//`) | ❌ | ❌ | ✅ | ✅ | Emulated with `math.floor(a/b)` |
| `<const>` variables | ❌ | ❌ | ❌ | ✅ | Type-checked only in 5.1-5.3 |
| `<close>` variables | ❌ | ❌ | ❌ | ✅ | Not available in older versions |

### Standard Library

| Library | 5.1 | 5.2 | 5.3 | 5.4 | Notes |
|---------|-----|-----|-----|-----|-------|
| `table.*` | ✅ | ✅ | ✅ | ✅ | Some functions moved in 5.2+ |
| `string.*` | ✅ | ✅ | ✅ | ✅ | — |
| `math.*` | ✅ | ✅ | ✅ | ✅ | — |
| `io.*` | ✅ | ✅ | ✅ | ✅ | — |
| `os.*` | ✅ | ✅ | ✅ | ✅ | — |
| `module()` | ✅ | ⚠️ | ❌ | ❌ | Deprecated in 5.2, removed in 5.3 |
| `setfenv/getfenv` | ✅ | ❌ | ❌ | ❌ | Replaced by `_ENV` |
| `bit32.*` | ❌ | ✅ | ✅ | ⚠️ | Deprecated in 5.3 (use bitwise ops) |
| `utf8.*` | ❌ | ❌ | ✅ | ✅ | — |
| `table.pack/unpack` | ⚠️ | ✅ | ✅ | ✅ | Use `unpack` in 5.1 |
| `table.move` | ❌ | ❌ | ✅ | ✅ | — |

Legend: ✅ Available | ❌ Not available | ⚠️ Deprecated/Changed

## Code Generation Differences

### Integers

**LuaNext:**

```lua
const count: integer = 42
const index: integer = #array
```

**Compiles to (5.3+):**

```lua
local count = 42
local index = #array
```

**Compiles to (5.1/5.2):**

```lua
-- Same output, but type is checked at compile time only
local count = 42
local index = #array
```

**Note:** In 5.1/5.2, `integer` is a type annotation only. In 5.3+, Lua uses actual integer type internally for performance.

### Bitwise Operators

**LuaNext:**

```lua
const a: integer = 12 & 10   -- AND
const b: integer = 12 | 10   -- OR
const c: integer = 12 ~ 10   -- XOR
const d: integer = ~5        -- NOT
const e: integer = 1 << 3    -- Left shift
const f: integer = 8 >> 2    -- Right shift
```

**Compiles to (5.3+):**

```lua
local a = 12 & 10
local b = 12 | 10
local c = 12 ~ 10
local d = ~5
local e = 1 << 3
local f = 8 >> 2
```

**Compiles to (5.2):**

```lua
-- Uses bit32 library
local a = bit32.band(12, 10)
local b = bit32.bor(12, 10)
local c = bit32.bxor(12, 10)
local d = bit32.bnot(5)
local e = bit32.lshift(1, 3)
local f = bit32.rshift(8, 2)
```

**Compiles to (5.1):**

```lua
-- Uses polyfill or external bit library
local a = bit.band(12, 10)
local b = bit.bor(12, 10)
local c = bit.bxor(12, 10)
local d = bit.bnot(5)
local e = bit.lshift(1, 3)
local f = bit.rshift(8, 2)
```

### Integer Division

**LuaNext:**

```lua
const result: number = 17 // 5  -- Floor division
```

**Compiles to (5.3+):**

```lua
local result = 17 // 5
```

**Compiles to (5.1/5.2):**

```lua
local result = math.floor(17 / 5)
```

### Table Operations

**LuaNext:**

```lua
const packed = table.pack(1, 2, 3, 4, 5)
const moved = table.move(source, 1, 5, 1, dest)
```

**Compiles to (5.2+):**

```lua
local packed = table.pack(1, 2, 3, 4, 5)
local moved = table.move(source, 1, 5, 1, dest)
```

**Compiles to (5.1):**

```lua
-- Polyfill for table.pack
local function pack(...)
    return {n = select("#", ...), ...}
end
local packed = pack(1, 2, 3, 4, 5)

-- Polyfill for table.move
local function move(a1, f, e, t, a2)
    a2 = a2 or a1
    for i = 0, e - f do
        a2[t + i] = a1[f + i]
    end
    return a2
end
local moved = move(source, 1, 5, 1, dest)
```

### Module System

**LuaNext:**

```lua
-- ES6-style imports/exports
import {add, multiply} from "./math-utils"
export function divide(a: number, b: number): number
    return a / b
end
```

**Compiles to (All versions):**

```lua
-- Uses require and module pattern
local math_utils = require("math-utils")
local add = math_utils.add
local multiply = math_utils.multiply

local M = {}

function M.divide(a, b)
    return a / b
end

return M
```

**Note:** Module compilation is the same across all Lua versions. LuaNext uses the return-table pattern, not `module()`.

## Cross-Version Compatibility Patterns

### Writing Portable Code

Use LuaNext features that work across all versions:

```lua
-- ✅ Portable: Basic types
const name: string = "Alice"
const count: number = 42

-- ✅ Portable: Functions
function add(a: number, b: number): number
    return a + b
end

-- ✅ Portable: Interfaces
interface Point
    x: number
    y: number
end

-- ✅ Portable: Arrays and tables
const numbers: number[] = {1, 2, 3}
const point: Point = {x = 1, y = 2}
```

### Avoid Version-Specific Features

If targeting multiple versions:

```lua
-- ❌ Avoid in portable code (5.3+ only)
const flags: integer = 0b1100
const result: integer = flags & 0b1010

-- ✅ Use instead (all versions)
const flags: number = 12
const result: number = bitwiseAnd(flags, 10)  -- Custom function
```

### Conditional Compilation

LuaNext doesn't support `#ifdef` style conditionals. Instead:

**Option 1: Configuration-based**

```yaml
# luanext-5.1.config.yaml
compilerOptions:
  target: "5.1"
  defines:
    USE_BIT_LIBRARY: true

# luanext-5.3.config.yaml
compilerOptions:
  target: "5.3"
  defines:
    USE_BIT_LIBRARY: false
```

**Option 2: Runtime detection**

```lua
-- Detect Lua version at runtime
const luaVersion = tonumber(_VERSION:match("%d+%.%d+"))

function bitwiseAnd(a: number, b: number): number
    if luaVersion >= 5.3 then
        return a & b
    else
        return bit32.band(a, b)
    end
end
```

## Testing Across Versions

### Using Multiple Lua Installations

```bash
# Test with Lua 5.1
lua5.1 output.lua

# Test with Lua 5.3
lua5.3 output.lua

# Test with Lua 5.4
lua5.4 output.lua
```

### Docker Testing

```dockerfile
# Dockerfile
FROM alpine:latest

RUN apk add --no-cache lua5.1 lua5.3 lua5.4

COPY dist/ /app/
WORKDIR /app

CMD ["lua5.4", "main.lua"]
```

### CI/CD Matrix Testing

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    lua-version: ['5.1', '5.2', '5.3', '5.4']
steps:
  - name: Setup Lua
    uses: leafo/gh-actions-lua@v9
    with:
      luaVersion: ${{ matrix.lua-version }}
  - name: Run tests
    run: lua test/all_tests.lua
```

## Performance Considerations

### Version Performance Characteristics

**Lua 5.1:**

- **Wide compatibility** — Runs on most platforms and embedded systems
- **Best for** — Legacy applications, maximum compatibility
- **Limitations** — No native integers, older standard library, no bitwise operators

**Lua 5.3:**

- **Good integer performance** — Native 64-bit integers
- **Bitwise operators** — Faster than library calls
- **Best for** — Numerical applications, bit manipulation

**Lua 5.4:**

- **Generational GC** — Better for applications with many short-lived objects
- **Best for** — Modern applications, long-running servers
- **Latest optimizations** — Most recent performance improvements

### Optimization by Target

**For Lua 5.1:**

```lua
-- Use number everywhere (no integer optimization)
function sum(arr: number[]): number
    local total: number = 0
    for i = 1, #arr do
        total = total + arr[i]
    end
    return total
end
```

**For Lua 5.3+:**

```lua
-- Use integer for loops and counters
function sum(arr: number[]): number
    local total: number = 0
    for i: integer = 1, #arr do
        total = total + arr[i]
    end
    return total
end
```

## Migration Between Versions

### Upgrading from 5.1 to 5.3+

**Changes needed:**

1. **Replace `bit` library with operators:**

```lua
-- Before (5.1)
const result = bit.band(a, b)

-- After (5.3+)
const result = a & b
```

1. **Replace `module()` with return pattern:**

```lua
-- Before (5.1)
module("mymodule")
function foo() end

-- After (all versions via LuaNext)
export function foo(): void end
```

1. **Use `table.pack` instead of `arg` table:**

```lua
-- Before (5.1)
function varargs(...)
    const args = arg
end

-- After (5.2+)
function varargs(...: any[])
    const args = table.pack(...)
end
```

### Downgrading from 5.4 to 5.1

**Changes needed:**

1. **Remove `<const>` and `<close>`** — LuaNext handles this automatically
2. **Avoid bitwise operators** — Use functions instead
3. **Test with polyfills** — Ensure compatibility functions work

## Summary

### Quick Decision Guide

```text
Need wide compatibility?  → Target 5.1
Need bitwise operators?   → Target 5.3+
Need latest features?     → Target 5.4
Building new project?     → Target 5.4
Legacy system support?    → Target 5.1
```

### Best Practices

1. **Choose target based on deployment environment**
2. **Test on actual target Lua version**
3. **Use portable LuaNext features when possible**
4. **Avoid version-specific Lua features**
5. **Document target version requirements**

## Next Steps

- [Migrating from Lua](migrating-from-lua.md) — Porting Lua code to LuaNext
- [Configuration](../reference/configuration.md) — All compiler options
- [CLI Reference](../reference/cli.md) — Command-line usage

## See Also

- [Migrating from Lua](migrating-from-lua.md) — Porting Lua code to LuaNext
- [Configuration](../reference/configuration.md) — All compiler options
- [CLI Reference](../reference/cli.md) — Command-line usage
- [Standard Library](../reference/standard-library.md) — Typed standard library
- [Lua Manual](https://www.lua.org/manual/) — Official Lua documentation
