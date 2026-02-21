# Codegen Targets

CodeGenStrategy trait, 6 Lua targets, and version-specific behavior.

## Overview

LuaNext supports 6 Lua targets. Each target implements the `CodeGenStrategy` trait, which abstracts version-specific code generation for operators, control flow, and runtime features.

**Files**: `crates/luanext-core/src/codegen/strategies/`

## CodeGenStrategy Trait

```rust
trait CodeGenStrategy {
    fn name(&self) -> &str;
    fn generate_bitwise_op(&self, op: BinaryOp, left: &str, right: &str) -> String;
    fn generate_integer_divide(&self, left: &str, right: &str) -> String;
    fn generate_continue(&self, label: Option<StringId>) -> String;
    fn generate_unary_bitwise_not(&self, operand: &str) -> String;
    fn emit_preamble(&self) -> Option<String>;
    fn supports_native_bitwise(&self) -> bool;
    fn supports_native_integer_divide(&self) -> bool;
    fn supports_goto(&self) -> bool;
    fn supports_native_continue(&self) -> bool { false }
    fn global_declaration_prefix(&self) -> Option<&str> { None }
}
```

## Target Comparison

| Feature | Lua 5.1 | Lua 5.2 | Lua 5.3 | Lua 5.4 | Lua 5.5 | LuaJIT |
| ------- | ------- | ------- | ------- | ------- | ------- | ------ |
| Bitwise operators | Preamble helpers | `bit32.*` | Native `& \| ~ << >>` | Native | Native | Preamble helpers |
| Integer division | `math.floor(a/b)` | `math.floor(a/b)` | Native `//` | Native `//` | Native `//` | `math.floor(a/b)` |
| Goto / Labels | No | Yes | Yes | Yes | Yes | Yes (extension) |
| Continue | No | `goto __continue` | `goto __continue` | `goto __continue` | Native `continue` | `goto __continue` |
| Global keyword | No prefix | No prefix | No prefix | No prefix | `global` prefix | No prefix |
| Preamble | Bitwise lib | None | None | None | None | Bitwise lib |

## Lua 5.1 (`lua51.rs`)

### Bitwise Operations

Emits a preamble defining helper functions:

```lua
local function _bit_band(a, b) ... end
local function _bit_bor(a, b) ... end
local function _bit_bxor(a, b) ... end
local function _bit_lshift(a, n) ... end
local function _bit_rshift(a, n) ... end
local function _bit_bnot(a) ... end
```

Usage: `a & b` → `_bit_band(a, b)`

### Integer Division

```lua
math.floor(a / b)
```

### Continue

Not supported on Lua 5.1. No goto available.

`[NOTE: BEHAVIOR]` On Lua 5.1 without goto support, `continue` statements may not compile correctly. The compiler should warn when targeting 5.1 with `continue` usage.

## Lua 5.2 (`lua52.rs`)

### Bitwise Operations

Uses the `bit32` standard library:

```lua
bit32.band(a, b)
bit32.bor(a, b)
bit32.bxor(a, b)
bit32.lshift(a, n)
bit32.rshift(a, n)
bit32.bnot(a)
```

`[NOTE: BEHAVIOR]` `bit32` is a Lua 5.2 standard library — NOT available in Lua 5.4 runtime. Tests targeting 5.2 can only verify output syntax, not runtime execution via mlua (which embeds Lua 5.4).

### Continue

Emulated via goto:

```lua
-- continue
goto __continue
-- at end of loop:
::__continue::
```

## Lua 5.3 (`lua53.rs`)

### Bitwise Operations

Native operators: `a & b`, `a | b`, `a ~ b`, `a << n`, `a >> n`, `~a`

### Integer Division

Native: `a // b`

### Continue

Via goto: `goto __continue` + `::__continue::` label

`[NOTE: BEHAVIOR]` Lua 5.3 uses the same native operators as Lua 5.4. The strategy implementations are nearly identical.

## Lua 5.4 (`lua54.rs`)

Same capabilities as Lua 5.3. This is the default target.

## Lua 5.5 (`lua55.rs`)

### Native Continue

```lua
continue    -- no goto hack needed
```

### Native Global

```lua
global x = 42    -- uses native global keyword
```

`[NOTE: BEHAVIOR]` Lua 5.5 is still in development. Testing with mlua requires version 0.11.6+. Some tests are `#[ignore]` pending Lua 5.5 support in the test runtime.

## LuaJIT (`luajit.rs`)

Based on Lua 5.1 with extensions:

- **Goto**: Supported (LuaJIT extension)
- **Bitwise**: Uses preamble helpers (same as Lua 5.1)
- **Integer division**: `math.floor(a/b)`
- **Continue**: Via goto (LuaJIT supports goto)

## Cross-References

- [Codegen Architecture](codegen-architecture.md) — how strategies are selected
- [Codegen Expressions](codegen-expressions.md) — operator codegen
- [Runtime Library](runtime-library.md) — Lua runtime snippets
