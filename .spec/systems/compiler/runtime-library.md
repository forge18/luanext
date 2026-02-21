# Runtime Library

Lua runtime snippets embedded in codegen output for class, enum, decorator, reflection, and bitwise operations.

## Overview

The `luanext-runtime` crate contains Lua code snippets that are embedded in generated output via `include_str!()`. These provide the runtime support for language features that require helper code.

**Files**: `crates/luanext-runtime/src/`

## Module Structure

| Module | Purpose |
| ------ | ------- |
| `class.rs` | Class instantiation and inheritance runtime |
| `enum_rt.rs` | Enum construction and rich enum support |
| `decorator.rs` | Decorator invocation runtime |
| `reflection.rs` | Reflection metadata and type registry |
| `module.rs` | Module loading and re-export helpers |
| `bitwise/` | Bitwise operator implementations for Lua 5.1/5.2 |

## Class Runtime

Provides the metatable setup for class instantiation:

- `__index` setup for method dispatch
- `__metatable` protection
- Inheritance chain via `setmetatable` with parent as fallback
- Constructor (`new`) factory function
- Abstract class guard (runtime error on instantiation attempt)

## Enum Runtime

### Simple Enums

Simple enums need no runtime support — they compile to plain tables.

### Rich Enums

Rich enum runtime provides:

- `__new` factory function for member construction
- `__values` list of all members
- `__byName` lookup table
- Built-in methods: `name()`, `ordinal()`, `equals()`
- `setmetatable` setup for method dispatch

## Decorator Runtime

Decorator invocation wrapping:

- Receives the decorated target and descriptor
- Returns the potentially modified target
- Supports decorator composition (multiple decorators applied in reverse order)

## Reflection Runtime

### Type Registry

```lua
__TypeRegistry = {}
```

Global registry mapping type IDs to metadata:

- Class name
- Base class reference
- Member list (properties, methods)
- Type ID (unique integer)

### Reflection Metadata

Generated per-class when reflection is enabled:

```lua
__TypeRegistry[TypeId] = {
    name = "ClassName",
    base = ParentTypeId,
    members = { ... },
}
```

### assertType()

Runtime type checking intrinsic:

```lua
function assertType(value, expectedTypeId)
    -- checks __TypeRegistry for type compatibility
end
```

### Reflection Modes

| Mode | Behavior |
| ---- | -------- |
| `None` | No reflection code generated |
| `Selective` | Only for modules importing `@std/reflection` |
| `Full` | All classes get reflection metadata |

## Bitwise Library

**Files**: `bitwise/`

Provides bitwise operations for Lua 5.1 and LuaJIT targets:

```lua
local function _bit_band(a, b) ... end
local function _bit_bor(a, b) ... end
local function _bit_bxor(a, b) ... end
local function _bit_bnot(a) ... end
local function _bit_lshift(a, n) ... end
local function _bit_rshift(a, n) ... end
```

These are injected as preamble code when targeting Lua 5.1 or LuaJIT.

`[NOTE: BEHAVIOR]` The bitwise preamble functions work on Lua 5.4 runtime (pure Lua implementation), so tests can execute them. This differs from `bit32.*` calls (Lua 5.2 target) which are NOT available in Lua 5.4.

## Module Runtime

Module loading helpers for bundle mode:

- Internal `__require` mechanism
- Module registry for avoiding duplicate loading
- Re-export forwarding

## Embedding

Runtime snippets are embedded via `include_str!()` in codegen:

```rust
const CLASS_RUNTIME: &str = include_str!("class.lua");
```

The codegen emits these snippets at the top of the generated output as needed, based on which language features are used in the source file.

## Cross-References

- [Codegen Targets](codegen-targets.md) — target-specific preamble injection
- [Classes](../features/classes.md) — class codegen using runtime
- [Enums](../features/enums.md) — enum codegen using runtime
- [Class Advanced](../features/class-advanced.md) — decorator codegen using runtime
