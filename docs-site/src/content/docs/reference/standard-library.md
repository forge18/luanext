---
title: Standard Library Reference
---

# Standard Library Reference

Complete reference for Lua standard library functions with LuaNext type annotations.

## Overview

LuaNext provides type definitions for the entire Lua standard library across all versions (5.1-5.4). These types enable autocomplete, type checking, and documentation in your editor.

**Coverage:**

- Global functions (print, type, tostring, etc.)
- string module
- table module
- math module
- io module
- os module
- debug module
- coroutine module
- package module
- Version-specific modules (bit32, utf8, etc.)

## Global Functions

### Output Functions

#### `print(...args: unknown): void`

Prints values to standard output.

```lua
print("Hello, World!")
print("Count:", 42)
print(1, 2, 3, 4, 5)
```

**Automatically converts values using `tostring()`.**

### Type Functions

#### `type(value: unknown): string`

Returns the type of a value as a string.

**Possible return values:**

- `"nil"` — nil value
- `"boolean"` — true or false
- `"number"` — numeric value
- `"string"` — text value
- `"function"` — function
- `"table"` — table/object
- `"thread"` — coroutine
- `"userdata"` — C userdata

```lua
print(type(42))          -- "number"
print(type("hello"))     -- "string"
print(type({}))          -- "table"
print(type(nil))         -- "nil"
```

#### `tostring(value: unknown): string`

Converts a value to a string.

```lua
const str: string = tostring(42)        -- "42"
const str2: string = tostring(true)     -- "true"
const str3: string = tostring({x = 1})  -- "table: 0x..."
```

**Calls `__tostring` metamethod if available.**

#### `tonumber(value: string | number, base?: number): number | nil`

Converts a string or number to a number.

```lua
const num: number | nil = tonumber("42")      -- 42
const hex: number | nil = tonumber("FF", 16)  -- 255
const bin: number | nil = tonumber("1010", 2) -- 10
const invalid: number | nil = tonumber("abc") -- nil
```

**Base:** Optional base for conversion (2-36), default is 10.

### Error Handling

#### `error(message: unknown, level?: number): never`

Raises an error with the given message.

```lua
if value < 0 then
    error("Value must be positive")
end

-- With stack level
error("Invalid argument", 2)  -- Report error at caller
```

**Never returns** — execution stops and error propagates.

#### `assert<T>(condition: T, message?: string): T`

Checks a condition, raises error if false.

```lua
const file = assert(io.open("data.txt"), "File not found")
-- If open returns nil, raises error with message

const value: number = assert(getValue())
-- Narrows type: value is guaranteed non-nil after assert
```

**Returns the condition value if true.**

### Table Access

#### `rawget<K, V>(table: {[K]: V}, key: K): V | nil`

Gets a table value, bypassing metamethods.

```lua
const value = rawget(obj, "key")  -- Ignores __index metamethod
```

#### `rawset<K, V>(table: {[K]: V}, key: K, value: V): {[K]: V}`

Sets a table value, bypassing metamethods.

```lua
rawset(obj, "key", "value")  -- Ignores __newindex metamethod
```

#### `rawequal<T>(v1: T, v2: T): boolean`

Compares values for equality, bypassing `__eq` metamethod.

```lua
const equal: boolean = rawequal(a, b)  -- True raw equality
```

### Iteration

#### `pairs<K, V>(table: {[K]: V}): iterator`

Returns an iterator for all key-value pairs.

```lua
const data = {x = 1, y = 2, z = 3}

for key, value in pairs(data) do
    print(key, value)
end
-- Output: x  1, y  2, z  3 (order not guaranteed)
```

#### `ipairs<V>(table: V[]): iterator`

Returns an iterator for sequential integer keys.

```lua
const arr: number[] = {10, 20, 30, 40}

for i, value in ipairs(arr) do
    print(i, value)
end
-- Output: 1  10, 2  20, 3  30, 4  40
```

**Stops at first nil value.**

#### `next<K, V>(table: {[K]: V}, key?: K): (K | nil, V | nil)`

Returns the next key-value pair (used internally by `pairs`).

```lua
const t = {a = 1, b = 2, c = 3}
const key, value = next(t, nil)  -- First pair
```

### Metatables

#### `setmetatable<T, M>(table: T, metatable: M | nil): T`

Sets or removes a metatable.

```lua
const obj = {}
const mt = {
    __index = function(t, key)
        return "default"
    end
}

setmetatable(obj, mt)
print(obj.any_key)  -- "default"
```

#### `getmetatable<T>(value: T): table | nil`

Gets the metatable of a value.

```lua
const mt = getmetatable(obj)

if mt ~= nil then
    print("Has metatable")
end
```

**Returns `__metatable` field if set** (metatable protection).

### Argument Handling

#### `select<T>(index: number | "#", ...args: T[]): unknown`

Selects arguments from a variable argument list.

```lua
-- Get count of arguments
const count: number = select("#", 1, 2, 3, 4, 5)  -- 5

-- Get arguments from index 3 onward
const a, b, c = select(3, 10, 20, 30, 40, 50)  -- 30, 40, 50
```

#### `unpack<T>(list: T[], i?: number, j?: number): ...T`

Returns multiple values from a table (Lua 5.1 name for `table.unpack`).

```lua
const arr = {1, 2, 3, 4, 5}
print(unpack(arr))        -- 1  2  3  4  5
print(unpack(arr, 2, 4))  -- 2  3  4
```

### Code Execution

#### `load(chunk: string | (() => string | nil), chunkname?: string, mode?: "t" | "b" | "bt", env?: table): (() => unknown) | nil, string | nil`

Loads a chunk from a string or function.

```lua
const func, err = load("return 2 + 2")
if func ~= nil then
    print(func())  -- 4
else
    print("Error:", err)
end
```

**Mode:**

- `"t"` — Text only (Lua source)
- `"b"` — Binary only (bytecode)
- `"bt"` — Both (default)

#### `loadfile(filename?: string, mode?: "t" | "b" | "bt", env?: table): (() => unknown) | nil, string | nil`

Loads a chunk from a file.

```lua
const func, err = loadfile("script.lua")
if func ~= nil then
    func()  -- Execute the loaded script
end
```

#### `dofile(filename?: string): ...unknown`

Loads and executes a file.

```lua
dofile("init.lua")  -- Load and run init.lua
```

**Deprecated:** Use `loadfile` for better error handling.

#### `pcall<T, R>(func: (...T) => ...R, ...args: T): (true, ...R) | (false, string)`

Calls a function in protected mode.

```lua
const success, result = pcall(function()
    return riskyOperation()
end)

if success then
    print("Result:", result)
else
    print("Error:", result)  -- result is error message
end
```

**Returns:** `true` + results on success, `false` + error message on failure.

#### `xpcall<T, R>(func: (...T) => ...R, errorHandler: (error: unknown) => unknown, ...args: T): ...unknown`

Calls a function with a custom error handler.

```lua
const result = xpcall(
    function() return operation() end,
    function(err)
        print("Error occurred:", err)
        print(debug.traceback())
        return nil
    end
)
```

### Garbage Collection

#### `collectgarbage(opt?: "collect" | "stop" | "restart" | "count" | "step" | "setpause" | "setstepmul" | "isrunning", arg?: number): unknown`

Controls the garbage collector.

```lua
-- Force full collection
collectgarbage("collect")

-- Get memory usage in KB
const memKB: number = collectgarbage("count")

-- Stop garbage collection
collectgarbage("stop")

-- Restart garbage collection
collectgarbage("restart")
```

## String Module

### `string.byte(s: string, i?: number, j?: number): ...number`

Returns byte values of characters.

```lua
print(string.byte("ABC"))        -- 65 (A)
print(string.byte("ABC", 2))     -- 66 (B)
print(string.byte("ABC", 1, 3))  -- 65 66 67
```

### `string.char(...bytes: number[]): string`

Converts bytes to a string.

```lua
const s: string = string.char(65, 66, 67)  -- "ABC"
```

### `string.find(s: string, pattern: string, init?: number, plain?: boolean): number | nil, number | nil, ...string`

Finds a pattern in a string.

```lua
const start, end = string.find("hello world", "world")  -- 7, 11
const i, j = string.find("test", "x")  -- nil, nil (not found)

-- Plain search (no patterns)
const pos = string.find("a.b.c", ".", 1, true)  -- 2 (finds literal dot)
```

### `string.format(format: string, ...args: unknown[]): string`

Formats a string (like C's printf).

```lua
const s: string = string.format("Hello, %s!", "World")  -- "Hello, World!"
const n: string = string.format("%d + %d = %d", 2, 3, 5)  -- "2 + 3 = 5"
const f: string = string.format("%.2f", 3.14159)  -- "3.14"
```

**Format specifiers:** `%s` (string), `%d` (integer), `%f` (float), `%x` (hex), etc.

### `string.gmatch(s: string, pattern: string): () => ...string`

Returns an iterator for pattern matches.

```lua
const text = "one two three"

for word in string.gmatch(text, "%w+") do
    print(word)
end
-- Output: one, two, three
```

### `string.gsub(s: string, pattern: string, repl: string | table | ((...: string) => string), n?: number): string, number`

Global substitution.

```lua
-- String replacement
const result = string.gsub("hello world", "world", "Lua")  -- "hello Lua", 1

-- Function replacement
const result2 = string.gsub("1 2 3", "%d", function(n)
    return tostring(tonumber(n) * 2)
end)  -- "2 4 6", 3

-- Table replacement
const replacements = {hello = "hi", world = "Lua"}
const result3 = string.gsub("hello world", "%w+", replacements)  -- "hi Lua", 2
```

### `string.len(s: string): number`

Returns string length in bytes.

```lua
const length: number = string.len("hello")  -- 5
```

**Equivalent to `#s` operator.**

### `string.lower(s: string): string`

Converts to lowercase.

```lua
const lower: string = string.lower("Hello World")  -- "hello world"
```

### `string.upper(s: string): string`

Converts to uppercase.

```lua
const upper: string = string.upper("Hello World")  -- "HELLO WORLD"
```

### `string.match(s: string, pattern: string, init?: number): ...string | nil`

Matches a pattern once.

```lua
const match: string | nil = string.match("abc123def", "%d+")  -- "123"
const a, b, c = string.match("2024-01-15", "(%d+)-(%d+)-(%d+)")  -- "2024", "01", "15"
```

### `string.rep(s: string, n: number, sep?: string): string`

Repeats a string n times.

```lua
const repeated: string = string.rep("ab", 3)  -- "ababab"
const with_sep: string = string.rep("x", 3, "-")  -- "x-x-x"
```

### `string.reverse(s: string): string`

Reverses a string.

```lua
const reversed: string = string.reverse("hello")  -- "olleh"
```

### `string.sub(s: string, i: number, j?: number): string`

Extracts a substring.

```lua
const sub: string = string.sub("hello", 2, 4)  -- "ell"
const from_start: string = string.sub("hello", 1, 3)  -- "hel"
const to_end: string = string.sub("hello", 3)  -- "llo"
```

**Negative indices count from the end:** `string.sub("hello", -3)` is `"llo"`.

## Table Module

### `table.concat(list: string[], sep?: string, i?: number, j?: number): string`

Concatenates table elements into a string.

```lua
const arr = {"a", "b", "c", "d"}
const str: string = table.concat(arr)           -- "abcd"
const csv: string = table.concat(arr, ", ")     -- "a, b, c, d"
const range: string = table.concat(arr, "-", 2, 3)  -- "b-c"
```

### `table.insert(list: T[], pos?: number, value: T): void`

Inserts an element into a table.

```lua
const arr: number[] = {1, 2, 3}

table.insert(arr, 4)      -- {1, 2, 3, 4} (append)
table.insert(arr, 2, 10)  -- {1, 10, 2, 3, 4} (insert at index 2)
```

### `table.remove(list: T[], pos?: number): T | nil`

Removes and returns an element.

```lua
const arr: number[] = {10, 20, 30, 40}

const last = table.remove(arr)     -- Returns 40, arr is {10, 20, 30}
const second = table.remove(arr, 2)  -- Returns 20, arr is {10, 30}
```

### `table.sort(list: T[], comp?: (a: T, b: T) => boolean): void`

Sorts a table in-place.

```lua
const arr: number[] = {3, 1, 4, 1, 5, 9, 2, 6}
table.sort(arr)  -- {1, 1, 2, 3, 4, 5, 6, 9}

-- Custom comparator (descending)
table.sort(arr, function(a, b) return a > b end)  -- {9, 6, 5, 4, 3, 2, 1, 1}
```

### `table.pack(...args: T[]): {n: number, [number]: T}` (Lua 5.2+)

Packs arguments into a table with count.

```lua
const packed = table.pack(1, 2, 3, 4, 5)
print(packed.n)     -- 5
print(packed[3])    -- 3
```

### `table.unpack(list: T[], i?: number, j?: number): ...T` (Lua 5.2+)

Unpacks a table into multiple values.

```lua
const arr = {10, 20, 30}
const a, b, c = table.unpack(arr)  -- a=10, b=20, c=30
```

**In Lua 5.1:** Use global `unpack()` function instead.

### `table.move(a1: T[], f: number, e: number, t: number, a2?: T[]): T[]` (Lua 5.3+)

Moves elements between tables.

```lua
const src = {1, 2, 3, 4, 5}
const dst = {10, 20, 30}

table.move(src, 1, 3, 2, dst)  -- dst becomes {10, 1, 2, 3, 30}
```

## Math Module

### Constants

- `math.pi` — π (3.14159265358979...)
- `math.huge` — Infinity (positive)
- `math.maxinteger` (5.3+) — Maximum integer value
- `math.mininteger` (5.3+) — Minimum integer value

### Trigonometric

```lua
math.sin(x: number): number      -- Sine
math.cos(x: number): number      -- Cosine
math.tan(x: number): number      -- Tangent
math.asin(x: number): number     -- Arcsine
math.acos(x: number): number     -- Arccosine
math.atan(y: number, x?: number): number  -- Arctangent
math.sinh(x: number): number     -- Hyperbolic sine (5.3+)
math.cosh(x: number): number     -- Hyperbolic cosine (5.3+)
math.tanh(x: number): number     -- Hyperbolic tangent (5.3+)
math.deg(x: number): number      -- Radians to degrees
math.rad(x: number): number      -- Degrees to radians
```

### Exponential/Logarithmic

```lua
math.exp(x: number): number      -- e^x
math.log(x: number, base?: number): number  -- Logarithm
math.sqrt(x: number): number     -- Square root
math.pow(x: number, y: number): number  -- x^y (deprecated, use ^)
```

### Rounding

```lua
math.abs(x: number): number      -- Absolute value
math.ceil(x: number): number     -- Round up
math.floor(x: number): number    -- Round down
math.modf(x: number): number, number  -- Integer and fractional parts
```

### Min/Max

```lua
math.max(...args: number[]): number  -- Maximum value
math.min(...args: number[]): number  -- Minimum value
```

### Random

```lua
math.random(m?: number, n?: number): number  -- Random number
math.randomseed(x: number): void             -- Set random seed
```

**Usage:**

- `math.random()` → Float in [0, 1)
- `math.random(n)` → Integer in [1, n]
- `math.random(m, n)` → Integer in [m, n]

### Integer Operations (Lua 5.3+)

```lua
math.tointeger(x: number): integer | nil  -- Convert to integer
math.type(x: number): "integer" | "float" | nil  -- Check number type
math.ult(m: integer, n: integer): boolean  -- Unsigned less than
```

## IO Module

### File Operations

```lua
io.open(filename: string, mode?: string): file | nil, string | nil
io.close(file?: file): boolean, string | nil
io.read(...formats: string[]): ...string | number | nil
io.write(...args: string[]): file | nil, string | nil
io.flush(): void
io.lines(filename?: string, ...formats: string[]): () => ...string | number | nil
io.input(file?: string | file): file
io.output(file?: string | file): file
io.tmpfile(): file
io.type(obj: unknown): "file" | "closed file" | nil
```

**File modes:** `"r"` (read), `"w"` (write), `"a"` (append), `"r+"` (read/write), `"w+"`, `"a+"`

### Example

```lua
const file = io.open("data.txt", "r")
if file ~= nil then
    const content = file:read("*all")
    file:close()
    print(content)
end
```

## OS Module

```lua
os.clock(): number                  -- CPU time used
os.date(format?: string, time?: number): string | table  -- Format date/time
os.difftime(t2: number, t1: number): number  -- Time difference
os.execute(command?: string): boolean, string, number | nil  -- Execute shell command
os.exit(code?: number | boolean, close?: boolean): never  -- Exit program
os.getenv(varname: string): string | nil  -- Get environment variable
os.remove(filename: string): boolean, string | nil  -- Delete file
os.rename(oldname: string, newname: string): boolean, string | nil  -- Rename file
os.setlocale(locale: string | nil, category?: string): string | nil  -- Set locale
os.time(date?: table): number  -- Get time
os.tmpname(): string  -- Generate temp filename
```

## Coroutine Module

```lua
coroutine.create(func: () => ...unknown): thread  -- Create coroutine
coroutine.resume(co: thread, ...args: unknown[]): boolean, ...unknown  -- Resume
coroutine.yield(...args: unknown[]): ...unknown  -- Yield
coroutine.status(co: thread): "running" | "suspended" | "normal" | "dead"  -- Status
coroutine.running(): thread | nil, boolean  -- Current coroutine
coroutine.wrap(func: () => ...unknown): (...args: unknown[]) => ...unknown  -- Wrap as function
coroutine.isyieldable(): boolean  -- Check if can yield (5.3+)
coroutine.close(co: thread): boolean, string | nil  -- Close coroutine (5.4+)
```

## Version-Specific Modules

### bit32 Module (Lua 5.2-5.3 only)

Bitwise operations library (deprecated in 5.3, use bitwise operators instead).

```lua
bit32.band(...args: number[]): number  -- Bitwise AND
bit32.bor(...args: number[]): number   -- Bitwise OR
bit32.bxor(...args: number[]): number  -- Bitwise XOR
bit32.bnot(x: number): number          -- Bitwise NOT
bit32.lshift(x: number, disp: number): number  -- Left shift
bit32.rshift(x: number, disp: number): number  -- Right shift
bit32.arshift(x: number, disp: number): number  -- Arithmetic right shift
```

### utf8 Module (Lua 5.3+)

UTF-8 string handling.

```lua
utf8.char(...codepoints: number[]): string  -- Codepoints to string
utf8.codes(s: string): () => number, number  -- Iterator over codepoints
utf8.codepoint(s: string, i?: number, j?: number): ...number  -- Extract codepoints
utf8.len(s: string, i?: number, j?: number): number | nil, number  -- UTF-8 length
utf8.offset(s: string, n: number, i?: number): number | nil  -- Byte offset of character
utf8.charpattern: string  -- Pattern matching single UTF-8 character
```

## See Also

- [Type System](../language/type-system.md) — Using standard library with types
- [Modules](../language/modules.md) — Importing and using modules
- [Lua Targets](../guides/lua-targets.md) — Version-specific standard library features
