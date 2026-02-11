# Code Generation Architecture

The LuaNext code generator transforms type-checked ASTs into executable Lua code with support for multiple Lua versions, source maps, bundling, and advanced optimizations. This document explains the architecture, strategies, and how to extend the code generator.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Lua Target Versions](#lua-target-versions)
4. [Type Erasure](#type-erasure)
5. [Statement Generation](#statement-generation)
6. [Expression Generation](#expression-generation)
7. [Class Generation](#class-generation)
8. [Function Generation](#function-generation)
9. [Module System](#module-system)
10. [Optimization Integration](#optimization-integration)
11. [Source Maps](#source-maps)
12. [Edge Cases](#edge-cases)
13. [Adding Features](#adding-features)
14. [Testing](#testing)

---

## Overview

### Goals

The code generator aims to:

- **Target multiple Lua versions**: Emit version-specific code for Lua 5.1-5.4
- **Type erasure**: Remove TypeScript-style types while preserving runtime behavior
- **Readable output**: Generate clean, idiomatic Lua code by default
- **Source map support**: Map generated Lua back to original LuaNext source
- **Bundle optimization**: Combine multiple modules with tree shaking and scope hoisting
- **Reflection support**: Optional runtime type information for advanced features

### Design Principles

1. **Strategy pattern**: Version-specific code generation encapsulated in strategies
2. **Emitter abstraction**: Separate output concerns from code generation logic
3. **Builder pattern**: Fluent API for configuring code generator
4. **Incremental emission**: Output written incrementally without building full string
5. **Format flexibility**: Support readable, compact, and minified output modes

### Code Generator API

**Location:** `/crates/luanext-core/src/codegen/`

```rust
use luanext_core::codegen::{CodeGeneratorBuilder, LuaTarget};

let interner = Arc::new(StringInterner::new());
let generator = CodeGeneratorBuilder::new(interner)
    .target(LuaTarget::Lua54)
    .source_map("main.luax".to_string())
    .optimization_level(OptimizationLevel::O2)
    .build();

let lua_code = generator.generate(&program);
```

---

## Architecture

### CodeGenerator Structure

**Location:** `/crates/luanext-core/src/codegen/mod.rs`

```rust
pub struct CodeGenerator {
    emitter: Emitter,                          // Output buffer + source maps
    target: LuaTarget,                         // Lua version (5.1-5.4)
    current_class_parent: Option<StringId>,    // For super calls
    uses_built_in_decorators: bool,            // Runtime library embedding
    mode: CodeGenMode,                         // Require vs Bundle
    exports: Vec<String>,                      // Module exports tracking
    has_default_export: bool,                  // Default export flag
    import_map: HashMap<String, String>,       // Import resolution (bundle)
    current_source_index: usize,               // Multi-source source maps
    interner: Arc<StringInterner>,             // Identifier resolution
    optimization_level: OptimizationLevel,     // O0-O3 optimization
    interface_default_methods: HashMap<...>,   // Interface defaults
    current_namespace: Option<Vec<String>>,    // Namespace path
    namespace_exports: Vec<(String, String)>,  // Namespace attachments
    next_type_id: u32,                         // Reflection type IDs
    registered_types: HashMap<String, u32>,    // Type registry
    reflection_mode: ReflectionMode,           // Selective/Full/None
    has_reflection_import: bool,               // @std/reflection usage
    strategy: Box<dyn CodeGenStrategy>,        // Lua version strategy
    enforce_access_modifiers: bool,            // Runtime access checks
    whole_program_analysis: Option<WPA>,       // Cross-module optimization
    reachable_exports: Option<HashSet<...>>,   // Tree shaking
    tree_shaking_enabled: bool,                // Tree shaking flag
    scope_hoisting_enabled: bool,              // Scope hoisting flag
}
```

**Key Components**:

1. **Emitter**: Handles output buffering, indentation, and source map tracking
2. **Strategy**: Encapsulates Lua version-specific code generation
3. **StringInterner**: Resolves `StringId` → `String` for identifiers
4. **Mode**: Controls module generation (separate files vs single bundle)

### Emitter Abstraction

**Location:** `/crates/luanext-core/src/codegen/emitter.rs`

The `Emitter` separates output concerns from code generation:

```rust
pub struct Emitter {
    output: String,                       // Generated code buffer
    indent_level: usize,                  // Current indentation depth
    indent_str: String,                   // Indentation string ("    ")
    source_map: Option<SourceMapBuilder>, // Source map builder
    output_format: OutputFormat,          // Readable/Compact/Minified
}

impl Emitter {
    pub fn write(&mut self, s: &str);          // Append text
    pub fn writeln(&mut self, s: &str);        // Append line
    pub fn indent(&mut self);                  // Increase indentation
    pub fn dedent(&mut self);                  // Decrease indentation
    pub fn write_indent(&mut self);            // Emit current indentation
}
```

**Output Formats**:

- **Readable** (default): 4-space indentation, newlines preserved
- **Compact**: 1-space indentation, minimal whitespace
- **Minified**: No indentation, no newlines (except in string literals)

### Builder Pattern

**Location:** `/crates/luanext-core/src/codegen/builder.rs`

The builder provides a fluent API for configuration:

```rust
pub struct CodeGeneratorBuilder {
    interner: Arc<StringInterner>,         // Required
    target: LuaTarget,                     // Default: Lua54
    source_map: Option<String>,            // Optional
    mode: CodeGenMode,                     // Default: Require
    optimization_level: OptimizationLevel, // Default: O0
    output_format: OutputFormat,           // Default: Readable
    whole_program_analysis: Option<WPA>,   // Optional
    reachable_exports: Option<HashSet<String>>, // Optional
    reflection_mode: ReflectionMode,       // Default: Selective
}
```

**Example Usage**:

```rust
let generator = CodeGeneratorBuilder::new(interner)
    .target(LuaTarget::Lua53)
    .source_map("src/main.luax".to_string())
    .bundle_mode("main".to_string())
    .optimization_level(OptimizationLevel::O2)
    .output_format(OutputFormat::Minified)
    .reflection_mode(ReflectionMode::Full)
    .build();
```

---

## Lua Target Versions

### Strategy Pattern

**Location:** `/crates/luanext-core/src/codegen/strategies/`

Each Lua version has a strategy implementing `CodeGenStrategy`:

```rust
pub trait CodeGenStrategy {
    fn name(&self) -> &str;
    fn generate_bitwise_op(&self, op: BinaryOp, left: &str, right: &str) -> String;
    fn generate_integer_divide(&self, left: &str, right: &str) -> String;
    fn generate_continue(&self, label: Option<StringId>) -> String;
    fn generate_unary_bitwise_not(&self, operand: &str) -> String;
    fn emit_preamble(&self) -> Option<String>;
    fn supports_native_bitwise(&self) -> bool;
    fn supports_native_integer_divide(&self) -> bool;
}
```

### Lua 5.1 Strategy

**Location:** `/crates/luanext-core/src/codegen/strategies/lua51.rs`

**Limitations**:
- No native bitwise operators
- No goto/labels
- No integer division operator

**Workarounds**:

1. **Bitwise Operations**: Emit helper functions in preamble
   ```lua
   -- Preamble (emitted once per file)
   local function _bit_band(a, b)
       local result = 0
       local bitval = 1
       while a > 0 and b > 0 do
           if a % 2 == 1 and b % 2 == 1 then
               result = result + bitval
           end
           bitval = bitval * 2
           a = math.floor(a / 2)
           b = math.floor(b / 2)
       end
       return result
   end
   -- Similar for bor, bxor, lshift, rshift, bnot
   ```

2. **Integer Division**: Use `math.floor(a / b)`

3. **Continue Statement**: Emulate with goto (not available, so error)

**Code Generation Example**:

```rust
// LuaNext: const x = a & b
strategy.generate_bitwise_op(BinaryOp::BitwiseAnd, "a", "b")
// Lua 5.1: _bit_band(a, b)
```

### Lua 5.2 Strategy

**Location:** `/crates/luanext-core/src/codegen/strategies/lua52.rs`

**Features**:
- `bit32` library for bitwise operations
- `goto` and labels
- Still no native integer division

**Code Generation**:

```rust
// LuaNext: const x = a & b
strategy.generate_bitwise_op(BinaryOp::BitwiseAnd, "a", "b")
// Lua 5.2: bit32.band(a, b)

// LuaNext: const y = a << 2
strategy.generate_bitwise_op(BinaryOp::ShiftLeft, "a", "2")
// Lua 5.2: bit32.lshift(a, 2)
```

### Lua 5.3+ Strategies

**Location:** `/crates/luanext-core/src/codegen/strategies/lua53.rs`, `lua54.rs`

**Features**:
- Native bitwise operators (`&`, `|`, `~`, `<<`, `>>`)
- Native integer division (`//`)
- `goto` and labels

**Code Generation**:

```rust
// LuaNext: const x = a & b
strategy.generate_bitwise_op(BinaryOp::BitwiseAnd, "a", "b")
// Lua 5.3+: (a & b)

// LuaNext: const y = a // b
strategy.generate_integer_divide("a", "b")
// Lua 5.3+: (a // b)
```

**Lua 5.4 Additions**:
- `const` variables (currently not emitted by LuaNext)
- To-be-closed variables (future feature)

### Version Selection

Default target is **Lua 5.4** for maximum feature support. Override via CLI:

```bash
luanext --target lua51 main.luax
luanext --target lua52 main.luax
luanext --target lua53 main.luax
luanext --target lua54 main.luax  # Default
```

---

## Type Erasure

### Principle

All type annotations are removed during code generation, preserving only runtime behavior:

```luanext
// LuaNext
function add(a: number, b: number): number {
    return a + b
}
const result: number = add(1, 2)
```

```lua
-- Generated Lua
local function add(a, b)
    return (a + b)
end
local result = add(1, 2)
```

### Type-Directed Optimizations

While types are erased, the type checker provides information used for optimization:

1. **Non-nil optimization** (O2+):
   ```luanext
   const obj: Object = { x: 1 }
   const y = obj?.x  // Object literal guaranteed non-nil
   ```
   ```lua
   local obj = { x = 1 }
   local y = obj.x  -- No nil check needed
   ```

2. **Devirtualization** (O3 with WPA):
   ```luanext
   class A { method() { return 1 } }
   const a: A = new A()  // Final type known
   a.method()
   ```
   ```lua
   -- Instead of a:method(), directly call A.method(a)
   local a = A.new()
   A.method(a)  -- Static dispatch
   ```

### Reflection Metadata

When reflection is enabled, type information is preserved as runtime tables:

```lua
-- Reflection mode: Full
Calculator.__ownFields = {
    { name = "value", type = "n", _flags = 1 },  -- n=number, flags=public
}

Calculator.__ownMethods = {
    { name = "add", params = "nn", ret = "n" },  -- two numbers -> number
}

__TypeRegistry["Calculator"] = 1  -- Type ID
__TypeIdToClass[1] = Calculator
```

**Reflection Modes**:

- **Selective** (default): Only emit for modules importing `@std/reflection`
- **Full**: Emit for all classes regardless of imports
- **None**: No reflection metadata (smallest output)

**Type Encoding** (compact):

| Type         | Code | Example                |
|--------------|------|------------------------|
| `number`     | `n`  | `n`                    |
| `string`     | `s`  | `s`                    |
| `boolean`    | `b`  | `b`                    |
| `table`      | `t`  | `t`                    |
| `function`   | `f`  | `f`                    |
| `void`/`nil` | `v`  | `v`                    |
| `any`        | `o`  | `o` (object)           |
| `Type[]`     | `[T]`| `[n]` (array of numbers)|
| `Type?`      | `?T` | `?s` (nullable string) |
| `A \| B`     | `A\|B`| `s\|n` (string or number)|

**Access Flags** (bitwise):

| Modifier    | Bit | Value |
|-------------|-----|-------|
| `public`    | 0   | 1     |
| `private`   | 1   | 2     |
| `protected` | 2   | 4     |
| `readonly`  | 3   | 8     |
| `static`    | 4   | 16    |

Example: `protected readonly` = 4 + 8 = 12

---

## Statement Generation

**Location:** `/crates/luanext-core/src/codegen/statements.rs`

### Variable Declarations

**LuaNext**:
```luanext
const x = 42
let y = "hello"
const z: boolean = true
```

**Generated Lua**:
```lua
local x = 42
local y = "hello"
local z = true
```

**Note**: `const` and `let` both become `local` (Lua doesn't distinguish mutability)

### Destructuring

**Array Destructuring**:

```luanext
const [a, b, ...rest] = [1, 2, 3, 4, 5]
```

```lua
local __temp = {1, 2, 3, 4, 5}
local a = __temp[1]
local b = __temp[2]
local rest = {table.unpack(__temp, 3)}
```

**Object Destructuring**:

```luanext
const { name, age } = person
```

```lua
local __temp = person
local name = __temp.name
local age = __temp.age
```

**Nested Destructuring**:

```luanext
const { user: { name, roles: [role1] } } = data
```

```lua
local __temp = data
local __temp_user = __temp.user
local name = __temp_user.name
local __temp_roles = __temp_user.roles
local role1 = __temp_roles[1]
```

### Function Declarations

**LuaNext**:
```luanext
function greet(name: string): string {
    return "Hello, " .. name
}
```

**Generated Lua**:
```lua
local function greet(name)
    return ("Hello, " .. name)
end
```

**Rest Parameters**:

```luanext
function sum(...numbers: number[]): number {
    local total = 0
    for _, n in ipairs(numbers) do
        total = total + n
    end
    return total
}
```

```lua
local function sum(...)
    local numbers = {...}
    local total = 0
    for _, n in ipairs(numbers) do
        total = (total + n)
    end
    return total
end
```

### Control Flow

**If/Else**:

```luanext
if x > 0 then
    print("positive")
elseif x < 0 then
    print("negative")
else
    print("zero")
end
```

```lua
if (x > 0) then
    print("positive")
elseif (x < 0) then
    print("negative")
else
    print("zero")
end
```

**While Loop**:

```luanext
while count < 10 do
    count = count + 1
end
```

```lua
while (count < 10) do
    count = (count + 1)
end
```

**Numeric For Loop**:

```luanext
for i = 1, 10, 2 do
    print(i)
end
```

```lua
for i = 1, 10, 2 do
    print(i)
end
```

**Generic For Loop with Destructuring**:

```luanext
for [x, y] in items do
    print(x, y)
end
```

```lua
for _, __item in ipairs(items) do
    local x = __item[1]
    local y = __item[2]
    print(x, y)
end
```

### Error Handling

**Try/Catch (pcall)**:

```luanext
try {
    riskyOperation()
} catch (e) {
    handleError(e)
}
```

```lua
-- try block
local __ok, __result = pcall(function()
    riskyOperation()
end)
if not __ok then
    local e = __result
    handleError(e)
end
```

**Try/Catch (xpcall with debug.traceback)** - O2+:

```lua
local __error
xpcall(function()
    riskyOperation()
end, debug.traceback)
if __error == nil then return end
local e = __error
handleError(e)
```

**Typed Catch**:

```luanext
try {
    operation()
} catch (e: NetworkError) {
    retryNetwork()
} catch (e: FileError) {
    logFileError(e)
}
```

Currently generates catch-all (type checking done at compile-time):

```lua
local __ok, __result = pcall(function()
    operation()
end)
if not __ok then
    local e = __result
    if false then
    elseif false then
    else
        -- All catches merged
        retryNetwork()
        logFileError(e)
    end
end
```

**Throw**:

```luanext
throw new Error("Something went wrong")
```

```lua
error(Error.new("Something went wrong"))
```

---

## Expression Generation

**Location:** `/crates/luanext-core/src/codegen/expressions.rs`

### Binary Operations

**Arithmetic**:

```luanext
const result = (a + b) * (c - d) / e
```

```lua
local result = (((a + b) * (c - d)) / e)
```

**Logical**:

```luanext
const check = x >= y and z < w
```

```lua
local check = ((x >= y) and (z < w))
```

**Bitwise** (version-dependent):

```luanext
const mask = a & 0xFF
const flags = b | 0x01
const toggle = c ~ 0xFFFF
const shifted = d << 2
```

```lua
-- Lua 5.3+
local mask = (a & 255)
local flags = (b | 1)
local toggle = (c ~ 65535)
local shifted = (d << 2)

-- Lua 5.2
local mask = bit32.band(a, 255)
local flags = bit32.bor(b, 1)
local toggle = bit32.bxor(c, 65535)
local shifted = bit32.lshift(d, 2)

-- Lua 5.1
local mask = _bit_band(a, 255)
local flags = _bit_bor(b, 1)
local toggle = _bit_bxor(c, 65535)
local shifted = _bit_lshift(d, 2)
```

**Null Coalescing**:

```luanext
const value = maybeNil ?? defaultValue
```

Simple expression (O0-O1):
```lua
local value = (maybeNil ~= nil and maybeNil or defaultValue)
```

Complex expression (O0-O1):
```lua
local value = (function()
    local __left = maybeNil
    return __left ~= nil and __left or defaultValue
end)()
```

Guaranteed non-nil (O2+):
```lua
local value = guaranteedNonNil  -- Null check optimized away
```

### Ternary/Conditional

```luanext
const result = condition ? trueValue : falseValue
```

```lua
local result = (condition and trueValue or falseValue)
```

**Caveat**: This works unless `trueValue` is `false` or `nil`. For correctness:

```lua
local result = (function()
    if condition then
        return trueValue
    else
        return falseValue
    end
end)()
```

### Optional Chaining

**Member Access**:

```luanext
const value = obj?.property
```

Simple object (O2+, guaranteed non-nil):
```lua
local value = obj.property
```

Simple object (O0-O1):
```lua
local value = (obj and obj.property or nil)
```

Complex object:
```lua
local value = (function()
    local __t = obj
    if __t then
        return __t.property
    else
        return nil
    end
end)()
```

**Index Access**:

```luanext
const item = arr?.[index]
```

```lua
local item = (arr and arr[index] or nil)
```

**Method Call**:

```luanext
const result = obj?.method(arg)
```

```lua
local result = (obj and obj:method(arg) or nil)
```

### Array Literals

**Simple**:

```luanext
const arr = [1, 2, 3]
```

```lua
local arr = {1, 2, 3}
```

**With Spread**:

```luanext
const combined = [1, 2, ...other, 3]
```

Pre-O2:
```lua
local combined = (function()
    local __arr = {}
    table.insert(__arr, 1)
    table.insert(__arr, 2)
    for _, __v in ipairs(other) do
        table.insert(__arr, __v)
    end
    table.insert(__arr, 3)
    return __arr
end)()
```

O2+ (with preallocation):
```lua
local combined = (function()
    local __arr = {nil, nil, nil}  -- Preallocate known size
    table.insert(__arr, 1)
    table.insert(__arr, 2)
    for _, __v in ipairs(other) do
        table.insert(__arr, __v)
    end
    table.insert(__arr, 3)
    return __arr
end)()
```

### Object Literals

**Simple**:

```luanext
const obj = { name: "John", age: 30 }
```

```lua
local obj = { name = "John", age = 30 }
```

**Computed Keys**:

```luanext
const key = "dynamic"
const obj = { [key]: value }
```

```lua
local key = "dynamic"
local obj = (function()
    local __obj = {}
    __obj[key] = value
    return __obj
end)()
```

**With Spread** (O2+ with preallocation):

```luanext
const merged = { a: 1, ...other, b: 2 }
```

```lua
local merged = (function()
    local __obj = { a = nil, b = nil }  -- Preallocate known keys
    __obj.a = 1
    for __k, __v in pairs(other) do
        __obj[__k] = __v
    end
    __obj.b = 2
    return __obj
end)()
```

### Template Literals

```luanext
const greeting = `Hello, ${name}! You are ${age} years old.`
```

```lua
local greeting = ("Hello, " .. tostring(name) .. "! You are " .. tostring(age) .. " years old.")
```

**Multiline** (dedented):

```luanext
const html = `
    <div>
        <h1>${title}</h1>
    </div>
`
```

```lua
local html = ("<div>\n    <h1>" .. tostring(title) .. "</h1>\n</div>")
```

### Match Expressions

```luanext
const result = match value {
    1 -> "one",
    2 -> "two",
    _ -> "other"
}
```

```lua
local result = (function()
    local __match_value = value
    if __match_value == 1 then
        return "one"
    elseif __match_value == 2 then
        return "two"
    else
        return "other"
    end
end)()
```

**Pattern Matching**:

```luanext
const point = { x: 10, y: 20 }
const result = match point {
    { x: 0, y: 0 } -> "origin",
    { x, y } if x == y -> "diagonal",
    { x, y } -> `(${x}, ${y})`
}
```

```lua
local point = { x = 10, y = 20 }
local result = (function()
    local __match_value = point
    if type(__match_value) == "table" and __match_value.x == 0 and __match_value.y == 0 then
        return "origin"
    elseif type(__match_value) == "table" and (x == y) then
        local x = __match_value.x
        local y = __match_value.y
        return "diagonal"
    elseif type(__match_value) == "table" then
        local x = __match_value.x
        local y = __match_value.y
        return ("(" .. tostring(x) .. ", " .. tostring(y) .. ")")
    else
        error("Non-exhaustive match")
    end
end)()
```

---

## Class Generation

**Location:** `/crates/luanext-core/src/codegen/classes.rs`

### Basic Class

```luanext
class Person {
    name: string
    age: number
}
```

```lua
local Person = {}
Person.__index = Person

function Person.new()
    local self = setmetatable({}, Person)
    return self
end

-- Type infrastructure
Person.__typeName = "Person"
Person.__typeId = 1
Person.__ancestors = {
    [1] = true,
}
```

### Constructor

```luanext
class Person {
    name: string
    age: number

    constructor(name: string, age: number) {
        self.name = name
        self.age = age
    }
}
```

```lua
local Person = {}
Person.__index = Person

function Person._init(self, name, age)
    self.name = name
    self.age = age
end

function Person.new(name, age)
    local self = setmetatable({}, Person)
    Person._init(self, name, age)
    return self
end

Person.__typeName = "Person"
Person.__typeId = 1
Person.__ancestors = {
    [1] = true,
}
```

**Why `_init` separation?**

This allows parent constructor calls (`super()`):

```luanext
class Student extends Person {
    studentId: number

    constructor(name: string, age: number, studentId: number) {
        super(name, age)
        self.studentId = studentId
    }
}
```

```lua
function Student._init(self, name, age, studentId)
    Person._init(self, name, age)  -- Call parent constructor
    self.studentId = studentId
end

function Student.new(name, age, studentId)
    local self = setmetatable({}, Student)
    Student._init(self, name, age, studentId)
    return self
end
```

### Methods

**Instance Method**:

```luanext
class Calculator {
    add(a: number, b: number): number {
        return a + b
    }
}
```

```lua
function Calculator:add(a, b)
    return (a + b)
end
```

**Static Method**:

```luanext
class Math {
    static abs(x: number): number {
        return x < 0 ? -x : x
    }
}
```

```lua
function Math.abs(x)
    return (x < 0 and -x or x)
end
```

**Note**: Static uses `.` (no implicit `self`), instance uses `:` (implicit `self`)

### Getters and Setters

```luanext
class Person {
    private _name: string

    get name(): string {
        return self._name
    }

    set name(value: string) {
        self._name = value
    }
}
```

```lua
function Person:get_name()
    return self._name
end

function Person:set_name(value)
    self._name = value
end
```

Usage:
```lua
local p = Person.new()
p:set_name("Alice")
local name = p:get_name()
```

### Inheritance

```luanext
class Animal {
    name: string

    constructor(name: string) {
        self.name = name
    }

    speak(): string {
        return "..."
    }
}

class Dog extends Animal {
    breed: string

    constructor(name: string, breed: string) {
        super(name)
        self.breed = breed
    }

    speak(): string {
        return "Woof!"
    }
}
```

```lua
local Animal = {}
Animal.__index = Animal

function Animal._init(self, name)
    self.name = name
end

function Animal.new(name)
    local self = setmetatable({}, Animal)
    Animal._init(self, name)
    return self
end

function Animal:speak()
    return "..."
end

Animal.__typeName = "Animal"
Animal.__typeId = 1
Animal.__ancestors = { [1] = true }

-- Dog class
local Dog = {}
Dog.__index = Dog

setmetatable(Dog, { __index = Animal })  -- Inheritance

function Dog._init(self, name, breed)
    Animal._init(self, name)  -- super(name)
    self.breed = breed
end

function Dog.new(name, breed)
    local self = setmetatable({}, Dog)
    Dog._init(self, name, breed)
    return self
end

function Dog:speak()
    return "Woof!"
end

Dog.__typeName = "Dog"
Dog.__typeId = 2
Dog.__ancestors = { [2] = true }

-- Inherit parent's ancestors
if Animal and Animal.__ancestors then
    for k, v in pairs(Animal.__ancestors) do
        Dog.__ancestors[k] = v
    end
end

Dog.__parent = Animal
```

### Super Calls

**Super Method**:

```luanext
class Dog extends Animal {
    speak(): string {
        const base = super.speak()
        return base .. " and Woof!"
    }
}
```

```lua
function Dog:speak()
    local base = Animal.speak(self)  -- super.speak() -> Animal.speak(self)
    return (base .. " and Woof!")
end
```

**Super Constructor** (already shown above via `_init` pattern)

### Abstract Classes

```luanext
abstract class Shape {
    abstract area(): number

    describe(): string {
        return `Area: ${self.area()}`
    }
}
```

```lua
local Shape = {}
Shape.__index = Shape

function Shape.new()
    local self = setmetatable({}, Shape)
    if self == nil or self.__typeName == "Shape" then
        error("Cannot instantiate abstract class 'Shape'")
    end
    return self
end

-- abstract methods not generated

function Shape:describe()
    return ("Area: " .. tostring(self:area()))
end

Shape.__typeName = "Shape"
Shape.__typeId = 1
Shape.__ancestors = { [1] = true }
```

**Subclass**:

```luanext
class Circle extends Shape {
    radius: number

    constructor(radius: number) {
        self.radius = radius
    }

    area(): number {
        return 3.14 * self.radius * self.radius
    }
}
```

```lua
function Circle.new(radius)
    local self = setmetatable({}, Circle)
    -- No abstract check here (Circle is concrete)
    Circle._init(self, radius)
    return self
end

function Circle:area()
    return ((3.14 * self.radius) * self.radius)
end
```

### Final Classes and Methods

```luanext
final class Immutable {
    final getValue(): number {
        return 42
    }
}
```

```lua
local Immutable = {}
Immutable.__index = Immutable

-- Mark class as final
Immutable.__final = true

function Immutable:getValue()
    return 42
end

-- Track final methods
Immutable.__finalMethods = { "getValue" }

-- Runtime check in subclass (if attempted)
if Immutable.__final then
    error("Cannot extend final class 'Immutable'")
end
```

### Operator Overloading

```luanext
class Vector {
    x: number
    y: number

    operator +(other: Vector): Vector {
        return new Vector(self.x + other.x, self.y + other.y)
    }

    operator ==(other: Vector): boolean {
        return self.x == other.x and self.y == other.y
    }
}
```

```lua
function Vector.__add(self, other)
    return Vector.new((self.x + other.x), (self.y + other.y))
end

function Vector.__eq(self, other)
    return ((self.x == other.x) and (self.y == other.y))
end

-- Attach metamethods
Vector.__metatable = {
    __add = Vector.__add,
    __eq = Vector.__eq,
}
```

**Supported Operators**:

| LuaNext Operator    | Lua Metamethod | Binary/Unary |
|---------------------|----------------|--------------|
| `+`                 | `__add`        | Binary       |
| `-`                 | `__sub`        | Binary       |
| `*`                 | `__mul`        | Binary       |
| `/`                 | `__div`        | Binary       |
| `%`                 | `__mod`        | Binary       |
| `^`                 | `__pow`        | Binary       |
| `..`                | `__concat`     | Binary       |
| `//`                | `__idiv`       | Binary       |
| `==`                | `__eq`         | Binary       |
| `<`                 | `__lt`         | Binary       |
| `<=`                | `__le`         | Binary       |
| `unary -`           | `__unm`        | Unary        |
| `#`                 | `__len`        | Unary        |
| `&` / `|` / `~` / `<<` / `>>` | `__band` / `__bor` / `__bxor` / `__shl` / `__shr` | Binary |
| `[]` (index)        | `__index`      | Special      |
| `[]=` (newindex)    | `__newindex`   | Special      |
| `()` (call)         | `__call`       | Special      |

---

## Function Generation

**Location:** `/crates/luanext-core/src/codegen/expressions.rs`, `statements.rs`

### Function Declarations

See [Statement Generation](#statement-generation) for details.

### Arrow Functions

```luanext
const add = (a: number, b: number) => a + b
```

```lua
local add = function(a, b)
    return (a + b)
end
```

**Block Body**:

```luanext
const process = (data: string) => {
    const trimmed = data.trim()
    return trimmed.toUpperCase()
}
```

```lua
local process = function(data)
    local trimmed = data:trim()
    return trimmed:toUpperCase()
end
```

### Closures and Upvalues

Lua handles closures naturally:

```luanext
function makeCounter() {
    let count = 0
    return () => {
        count = count + 1
        return count
    }
}
```

```lua
local function makeCounter()
    local count = 0
    return function()
        count = (count + 1)
        return count
    end
end
```

### Generic Specialization

Generic functions are specialized at call sites (optimization pass, not codegen):

```luanext
function identity<T>(x: T): T {
    return x
}

const num = identity(42)       // T = number
const str = identity("hello")  // T = string
```

**Without Specialization** (O0-O1):
```lua
local function identity(x)
    return x
end

local num = identity(42)
local str = identity("hello")
```

**With Specialization** (O2+):
```lua
-- Specialized versions generated
local function identity_number(x)
    return x
end

local function identity_string(x)
    return x
end

local num = identity_number(42)
local str = identity_string("hello")
```

This enables devirtualization and inlining.

---

## Module System

**Location:** `/crates/luanext-core/src/codegen/modules.rs`

### Require Mode (Default)

Each module is a separate file with `require()` calls:

**main.luax**:
```luanext
import { add, subtract } from "./math"

const result = add(10, 5)
print(result)
```

**main.lua** (generated):
```lua
local _mod = require("./math")
local add, subtract = _mod.add, _mod.subtract

local result = add(10, 5)
print(result)
```

**math.luax**:
```luanext
export function add(a: number, b: number): number {
    return a + b
}

export function subtract(a: number, b: number): number {
    return a - b
}
```

**math.lua** (generated):
```lua
local function add(a, b)
    return (a + b)
end

local function subtract(a, b)
    return (a - b)
end

local M = {}
M.add = add
M.subtract = subtract
return M
```

### Bundle Mode

All modules combined into single file with custom module loader:

**Command**:
```bash
luanext --bundle main.luax -o bundle.lua
```

**bundle.lua**:
```lua
-- LuaNext Bundle
-- Generated by LuaNext compiler

-- Module prelude (custom require implementation)
local __modules = {}
local __cache = {}

function __require(module_id)
    if __cache[module_id] then
        return __cache[module_id]
    end

    local loader = __modules[module_id]
    if not loader then
        error("Module not found: " .. module_id)
    end

    local result = loader()
    __cache[module_id] = result
    return result
end

-- Module: ./math
__modules["./math"] = function()
    local function add(a, b)
        return (a + b)
    end

    local function subtract(a, b)
        return (a - b)
    end

    local M = {}
    M.add = add
    M.subtract = subtract
    return M
end

-- Module: ./main
__modules["./main"] = function()
    local _mod = __require("./math")
    local add, subtract = _mod.add, _mod.subtract

    local result = add(10, 5)
    print(result)
end

-- Execute entry point
__require("./main")
```

### Tree Shaking

Eliminates unused exports in bundle mode:

**math.luax**:
```luanext
export function add(a: number, b: number): number {
    return a + b
}

export function subtract(a: number, b: number): number {
    return a - b  // Never imported
}

export function multiply(a: number, b: number): number {
    return a * b  // Never imported
}
```

**main.luax**:
```luanext
import { add } from "./math"

print(add(1, 2))
```

**bundle.lua (with tree shaking)**:
```lua
-- Module: ./math
__modules["./math"] = function()
    local function add(a, b)
        return (a + b)
    end
    -- subtract and multiply omitted (dead code)

    local M = {}
    M.add = add
    return M
end

-- Module: ./main
__modules["./main"] = function()
    local _mod = __require("./math")
    local add = _mod.add

    print(add(1, 2))
end

__require("./main")
```

**Enable Tree Shaking**:
```bash
luanext --bundle --tree-shaking main.luax
```

### Scope Hoisting

Hoists top-level declarations to bundle scope, eliminating module function wrappers:

**Without Scope Hoisting**:
```lua
__modules["./utils"] = function()
    local function helper() return 42 end
    local M = {}
    M.helper = helper
    return M
end

__modules["./main"] = function()
    local _mod = __require("./utils")
    local helper = _mod.helper
    print(helper())
end
```

**With Scope Hoisting**:
```lua
-- Hoisted declarations (scope hoisting)
local function utils__helper() return 42 end  -- Mangled name

-- Module: ./utils (skipped - fully hoisted)

-- Module: ./main
__modules["./main"] = function()
    print(utils__helper())  -- Direct call, no require
end

__require("./main")
```

**Benefits**:
- Eliminates `require()` overhead
- Enables cross-module inlining
- Reduces closure allocations

**Enable Scope Hoisting** (default in bundle mode):
```bash
luanext --bundle main.luax  # Enabled by default
luanext --bundle --no-scope-hoisting main.luax  # Disable
```

### Default Exports

```luanext
// config.luax
export default {
    apiUrl: "https://api.example.com",
    timeout: 5000
}
```

```lua
-- config.lua
local _default = {
    apiUrl = "https://api.example.com",
    timeout = 5000,
}

local M = {}
M.default = _default
return M
```

**Importing**:

```luanext
import config from "./config"

print(config.apiUrl)
```

```lua
local config = require("./config")

print(config.apiUrl)
```

### Re-exports

```luanext
// barrel.luax
export { add, subtract } from "./math"
export { sort, filter } from "./array"
```

```lua
-- barrel.lua
local _mod = require("./math")
local add, subtract = _mod.add, _mod.subtract

local _mod = require("./array")
local sort, filter = _mod.sort, _mod.filter

local M = {}
M.add = add
M.subtract = subtract
M.sort = sort
M.filter = filter
return M
```

---

## Optimization Integration

The code generator integrates with the optimizer at multiple levels:

### O0: No Optimizations

Direct transpilation with minimal transformations:

```luanext
const x = 2 + 3
```

```lua
local x = (2 + 3)  -- No constant folding
```

### O1: Basic Optimizations

Constants folded before code generation:

```luanext
const x = 2 + 3
```

```lua
local x = 5  -- Folded by optimizer
```

### O2: Standard Optimizations

1. **Null Coalescing Optimization**:
   ```luanext
   const obj = { x: 1 }
   const y = obj?.x
   ```

   ```lua
   local obj = { x = 1 }
   local y = obj.x  -- No nil check (guaranteed non-nil)
   ```

2. **Table Preallocation**:
   ```luanext
   const arr = [1, ...items, 2]
   ```

   ```lua
   local arr = (function()
       local __arr = {nil, nil}  -- Preallocate 2 known elements
       table.insert(__arr, 1)
       for _, __v in ipairs(items) do
           table.insert(__arr, __v)
       end
       table.insert(__arr, 2)
       return __arr
   end)()
   ```

3. **Try/Catch Optimization**:
   - Use `xpcall` with `debug.traceback` instead of `pcall`
   - Provides stack traces for debugging

### O3: Advanced Optimizations

1. **Devirtualization** (with whole-program analysis):
   ```luanext
   class A { method() { return 1 } }
   final class B extends A {}

   const b: B = new B()
   b.method()  // Type known to be B
   ```

   ```lua
   -- Direct static call instead of dynamic dispatch
   local b = B.new()
   B.method(b)  -- Instead of b:method()
   ```

2. **Generic Specialization**:
   ```luanext
   function identity<T>(x: T): T { return x }

   identity(42)
   identity("hello")
   ```

   ```lua
   local function identity_number(x) return x end
   local function identity_string(x) return x end

   identity_number(42)
   identity_string("hello")
   ```

3. **Function Inlining**:
   ```luanext
   function add(a, b) { return a + b }
   const result = add(x, y)
   ```

   ```lua
   local result = (x + y)  -- Inlined
   ```

---

## Source Maps

**Location:** `/crates/luanext-core/src/codegen/sourcemap.rs`

### Purpose

Source maps map generated Lua code back to original LuaNext source for debugging:

```
Generated Lua (line 42) -> Original LuaNext (line 15, column 8)
```

### Structure

Follows [Source Map v3 specification](https://sourcemaps.info/spec.html):

```json
{
  "version": 3,
  "file": "output.lua",
  "sources": ["main.luax"],
  "mappings": "AAAA,CAAC;AACD...",
  "names": ["myVariable", "myFunction"]
}
```

### Generation

Enabled via builder or CLI:

```rust
let generator = CodeGeneratorBuilder::new(interner)
    .source_map("main.luax".to_string())
    .build();
```

```bash
luanext --source-map main.luax
```

**Output**:
- `main.lua` - Generated Lua code
- `main.lua.map` - Source map JSON

### Mappings

Each mapping tracks:

```rust
struct Mapping {
    generated_line: usize,    // Line in generated Lua
    generated_column: usize,  // Column in generated Lua
    source_index: usize,      // Index into sources array
    source_line: usize,       // Line in original LuaNext
    source_column: usize,     // Column in original LuaNext
    name_index: Option<usize>, // Index into names array
}
```

Encoded using Base64 VLQ for compact representation.

### Bundle Source Maps

For bundles, multi-source source maps track all modules:

```json
{
  "version": 3,
  "file": "bundle.lua",
  "sources": ["main.luax", "math.luax", "utils.luax"],
  "mappings": "..."
}
```

Each module's mappings are merged with proper offsets.

---

## Edge Cases

### Nil Safety

LuaNext enforces nil safety at compile-time, but generated Lua must handle nil correctly:

**Non-nullable Type**:
```luanext
const x: string = "hello"
x.toUpperCase()  // Safe
```

```lua
local x = "hello"
x:toUpperCase()  -- No nil check
```

**Nullable Type**:
```luanext
const x: string? = maybeString()
if x != nil then
    x.toUpperCase()
end
```

```lua
local x = maybeString()
if (x ~= nil) then
    x:toUpperCase()
end
```

**Optional Chaining** (automatic nil handling):
```luanext
const result = obj?.method?.()
```

```lua
local result = (obj and obj.method and obj:method() or nil)
```

### Union Type Handling

Union types are erased, relying on Lua's dynamic typing:

```luanext
function process(x: string | number): string {
    if type(x) == "string" then
        return x
    else
        return tostring(x)
    end
}
```

```lua
local function process(x)
    if type(x) == "string" then
        return x
    else
        return tostring(x)
    end
end
```

Type guards (`type(x) == "string"`) are preserved in generated code.

### Pattern Match Compilation

Complex patterns compile to nested if/else:

```luanext
const result = match value {
    [1, 2, x] -> x,
    [a, b] -> a + b,
    _ -> 0
}
```

```lua
local result = (function()
    local __match_value = value
    if type(__match_value) == "table" and __match_value[1] == 1 and __match_value[2] == 2 then
        local x = __match_value[3]
        return x
    elseif type(__match_value) == "table" then
        local a = __match_value[1]
        local b = __match_value[2]
        return (a + b)
    else
        return 0
    end
end)()
```

### Varargs Handling

Lua's `...` has special semantics:

```luanext
function varargs(...args: any[]) {
    print(args)
}
```

```lua
local function varargs(...)
    local args = {...}  -- Capture to table
    print(args)
end
```

**Why Capture?**

Accessing `...` directly in nested functions doesn't work. Capturing to a table makes it available everywhere.

### String Escaping

Generated Lua strings must escape special characters:

```luanext
const path = "C:\Users\name"
```

```lua
local path = "C:\\Users\\name"  -- Backslash escaped
```

Template literals preserve multiline:

```luanext
const msg = `Line 1
Line 2`
```

```lua
local msg = "Line 1\nLine 2"
```

---

## Adding Features

### 1. Extend AST

Add new node type to `/crates/luanext-parser/src/ast/`:

```rust
// In ast/statement.rs
pub enum Statement<'arena> {
    // ... existing variants
    Using(UsingStatement<'arena>),
}

pub struct UsingStatement<'arena> {
    pub resource: Expression<'arena>,
    pub body: Block<'arena>,
    pub span: Span,
}
```

### 2. Update Parser

Add parsing logic:

```rust
// In parser/statement.rs
impl<'a, 'arena> Parser<'a, 'arena> {
    fn parse_using_statement(&mut self) -> Result<Statement<'arena>, ParserError> {
        self.consume(TokenKind::Using, "Expected 'using'")?;
        let resource = self.parse_expression()?;
        self.consume(TokenKind::Do, "Expected 'do'")?;
        let body = self.parse_block()?;
        self.consume(TokenKind::End, "Expected 'end'")?;

        Ok(Statement::Using(UsingStatement {
            resource,
            body,
            span: /* ... */,
        }))
    }
}
```

### 3. Type Check

Add type checking logic:

```rust
// In typechecker (external crate)
fn check_using_statement(&mut self, stmt: &UsingStatement) -> Result<(), TypeError> {
    let resource_type = self.check_expression(&stmt.resource)?;
    // Ensure resource has __close metamethod (Lua 5.4)
    self.check_has_close_method(resource_type)?;
    self.check_block(&stmt.body)?;
    Ok(())
}
```

### 4. Generate Code

Add code generation in `/crates/luanext-core/src/codegen/statements.rs`:

```rust
impl CodeGenerator {
    pub fn generate_using_statement(&mut self, stmt: &UsingStatement) {
        // Only for Lua 5.4 (has to-be-closed variables)
        if self.target == LuaTarget::Lua54 {
            self.write_indent();
            self.write("local <close> __resource = ");
            self.generate_expression(&stmt.resource);
            self.writeln("");
            self.generate_block(&stmt.body);
        } else {
            // Fallback: manual pcall cleanup
            self.write_indent();
            self.writeln("do");
            self.indent();
            self.write_indent();
            self.write("local __resource = ");
            self.generate_expression(&stmt.resource);
            self.writeln("");

            self.write_indent();
            self.writeln("local __ok, __err = pcall(function()");
            self.indent();
            self.generate_block(&stmt.body);
            self.dedent();
            self.write_indent();
            self.writeln("end)");

            self.write_indent();
            self.writeln("if __resource.__close then");
            self.indent();
            self.write_indent();
            self.writeln("__resource:__close()");
            self.dedent();
            self.write_indent();
            self.writeln("end");

            self.write_indent();
            self.writeln("if not __ok then error(__err) end");
            self.dedent();
            self.write_indent();
            self.writeln("end");
        }
    }
}
```

### 5. Add Strategy Method (If Version-Specific)

Update `CodeGenStrategy` trait if needed:

```rust
pub trait CodeGenStrategy {
    // ... existing methods
    fn supports_to_be_closed(&self) -> bool {
        false  // Default: not supported
    }
}

impl Lua54Strategy {
    fn supports_to_be_closed(&self) -> bool {
        true  // Lua 5.4 supports <close>
    }
}
```

### 6. Test

Add tests in `/crates/luanext-core/src/codegen/mod.rs`:

```rust
#[test]
fn test_using_statement_lua54() {
    let source = r#"
        using file = open("data.txt") do
            print(file.read())
        end
    "#;
    let output = generate_code_with_target(source, LuaTarget::Lua54);
    assert!(output.contains("local <close> __resource = open(\"data.txt\")"));
}

#[test]
fn test_using_statement_lua53() {
    let source = r#"
        using file = open("data.txt") do
            print(file.read())
        end
    "#;
    let output = generate_code_with_target(source, LuaTarget::Lua53);
    // Should use pcall fallback
    assert!(output.contains("pcall(function()"));
    assert!(output.contains("__resource:__close()"));
}
```

---

## Testing

### Unit Tests

Located in `/crates/luanext-core/src/codegen/mod.rs`:

```rust
#[test]
fn test_generate_variable_declaration() {
    let source = "const x = 42";
    let output = generate_code(source);
    assert!(output.contains("local x = 42"));
}
```

### Snapshot Tests

Using `insta` crate for snapshot testing:

```rust
#[test]
fn test_snapshot_class_with_inheritance() {
    let source = r#"
        class Animal {
            speak() { return "..." }
        }
        class Dog extends Animal {
            speak() { return "Woof!" }
        }
    "#;
    let output = generate_code(source);
    insta::assert_snapshot!(output);
}
```

Run with:
```bash
cargo test
cargo insta review  # Review snapshot changes
```

### Roundtrip Tests

Verify generated Lua is valid:

```rust
#[test]
fn test_roundtrip_control_flow() {
    let source = r#"
        if x > 0 then
            print("positive")
        end
    "#;
    let generated = generate_code(source);

    // Parse generated Lua (should not panic)
    let arena = Bump::new();
    let interner = StringInterner::new();
    let lexer = Lexer::new(&generated, &interner);
    let tokens = lexer.tokenize().expect("Roundtrip lexing failed");
    let parser = Parser::new(tokens, &interner, &common, &arena);
    let _program = parser.parse().expect("Roundtrip parsing failed");
}
```

### Integration Tests

Located in `/crates/luanext-cli/tests/cli_features_tests.rs`:

Test end-to-end compilation:

```rust
#[test]
fn test_compile_with_source_map() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_file = temp_dir.path().join("main.luax");
    std::fs::write(&input_file, "const x = 42\nprint(x)").unwrap();

    let output = Command::new("luanext")
        .arg("--source-map")
        .arg(&input_file)
        .output()
        .expect("Failed to execute luanext");

    assert!(output.status.success());

    let map_file = temp_dir.path().join("main.lua.map");
    assert!(map_file.exists());

    let map_content = std::fs::read_to_string(&map_file).unwrap();
    let map: SourceMap = serde_json::from_str(&map_content).unwrap();
    assert_eq!(map.version, 3);
    assert_eq!(map.sources, vec!["main.luax"]);
}
```

### Cross-Version Testing

Test all Lua targets:

```rust
#[test]
fn test_all_targets() {
    let source = "const x = a & b";

    for target in &[LuaTarget::Lua51, LuaTarget::Lua52, LuaTarget::Lua53, LuaTarget::Lua54] {
        let output = generate_code_with_target(source, *target);
        // Verify version-specific output
        match target {
            LuaTarget::Lua51 => assert!(output.contains("_bit_band")),
            LuaTarget::Lua52 => assert!(output.contains("bit32.band")),
            LuaTarget::Lua53 | LuaTarget::Lua54 => assert!(output.contains("(a & b)")),
        }
    }
}
```

### Runtime Testing

Execute generated Lua with actual Lua interpreters:

```bash
#!/bin/bash
luanext --target lua53 test.luax -o test.lua
lua5.3 test.lua  # Run and verify output
```

---

Agent is calibrated...
