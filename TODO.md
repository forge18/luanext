# LuaNext TODO

## Low Priority

### Language Features

#### Runtime Type Assertions via `assertType<T>()`

**Rationale:** Currently `x as Type` is purely compile-time — the type annotation is erased during codegen with no runtime validation. This is unsafe when dealing with external data (user input, API responses, deserialized data). A compiler-intrinsic `assertType<T>(value)` function would emit runtime `type()` checks and throw on mismatch, providing a safe bridge between untyped external data and the type system.

**Estimated Effort:** 2-3 weeks

**Syntax:**

```luanext
// Primitive type assertion — emits runtime type() check
const name = assertType<string>(input)
// Compiled: if type(input) ~= "string" then error("Type assertion failed: expected string, got " .. type(input)) end

// Class/interface assertion — emits table + metatable check
const user = assertType<User>(data)
// Compiled: if type(data) ~= "table" then error(...) end

// Union types — emits compound check
const id = assertType<string | number>(value)
// Compiled: if type(value) ~= "string" and type(value) ~= "number" then error(...) end

// Optional types
const name = assertType<string?>(input)
// Compiled: if input ~= nil and type(input) ~= "string" then error(...) end

// Use in function boundaries for safe parsing
function parseConfig(raw: unknown): Config {
  const data = assertType<table>(raw)
  const host = assertType<string>(data.host)
  const port = assertType<number>(data.port)
  return { host, port }
}
```

**Current State:**

- `TypeAssertion` (`as`) exists but is compile-time only — codegen just emits the inner expression (`crates/luanext-core/src/codegen/expressions.rs:434-436`)
- `assert<T>` already declared in stdlib as Lua's truthiness assert (`crates/luanext-typechecker/src/stdlib/builtins.d.tl:46`)
- `type()` function available in all Lua versions (`builtins.d.tl:18`)
- `typeof` narrowing already works in the type checker (`crates/luanext-typechecker/src/visitors/narrowing.rs:146-156`)
- Type predicate system (`x is T`) exists for user-defined type guards (`crates/luanext-parser/src/ast/types.rs:61`)
- `error()` function available for throwing (`builtins.d.tl:39`)

**Benefits:**

- Safe bridge between `unknown`/external data and typed code
- Clear error messages at runtime: "Type assertion failed: expected string, got number"
- Type narrowing after assertion — subsequent code sees the narrowed type
- Zero overhead in production if compiled with assertions stripped (future optimization flag)
- Complements existing `as` (compile-time) — `assertType` is the runtime counterpart

**Design Decisions:**

- **Name**: `assertType` (not `assert`, which conflicts with Lua's built-in)
- **Intrinsic**: Recognized by the compiler, not a library function — allows generating optimal code per type
- **Return type**: `assertType<T>(value: unknown): T` — returns the value with narrowed type
- **Error behavior**: Throws via `error()` with descriptive message including expected vs actual type

##### Phase 1: Intrinsic Recognition ✅

- [x] Register `assertType` as a compiler intrinsic in the type checker (not in stdlib)
- [x] Validate: exactly one type argument required, exactly one value argument
- [x] Return type is the type argument `T`
- [x] Value argument accepts `unknown` or any supertype of `T`
- [x] Report error if type argument is not a checkable type (e.g., generic type parameters)

**Implementation:** `crates/luanext-typechecker/src/visitors/inference.rs:215-222, 2520-2597`

##### Phase 2: Codegen for Primitive Types ✅

- [x] Recognize `assertType` calls in codegen expression handler
- [x] Emit `type()` check for primitives: `string`, `number`, `boolean`, `nil`, `table`
- [x] Generate descriptive error message: `"Type assertion failed: expected <T>, got " .. type(value)`
- [x] Emit the value expression as the result (assertion returns the value)
- [x] Handle `integer` type (Lua 5.3+: `math.type(x) == "integer"`, Lua 5.1/5.2: `type(x) == "number" and x % 1 == 0`)

**Implementation:** `crates/luanext-core/src/codegen/expressions.rs:119-131, 903-1054`

**Tests:**

- Type checker: `crates/luanext-typechecker/tests/assert_type_tests.rs` (7 tests)
- Codegen: `crates/luanext-core/tests/codegen_assert_type_tests.rs` (9 tests)

##### Phase 3: Codegen for Complex Types ✅

- [x] **Literal types**: `assertType<"hello">(x)` emits `x == "hello"` value equality check (including nil, boolean, string, number)
- [x] **Union types**: `assertType<string | number>(x)` emits compound `type()` check with `or`
- [x] **Optional types (Nullable)**: `assertType<string?>(x)` emits `__val ~= nil and type(__val) == "string"`
- [ ] **Class types**: `assertType<MyClass>(x)` emits table check + optional metatable/constructor check (deferred)
- [ ] **Interface types**: structural check — verify required properties exist (optional: deep check behind a flag) (deferred)

**Implementation:** `crates/luanext-core/src/codegen/expressions.rs:996-1143`

**Tests:**

- Type checker: `crates/luanext-typechecker/tests/assert_type_tests.rs` (11 tests)
- Codegen: `crates/luanext-core/tests/codegen_assert_type_tests.rs` (15 tests)

##### Phase 4: Type Narrowing Integration

- [ ] After `assertType<T>(x)`, narrow `x` to type `T` in the enclosing scope (not just the return value)
- [ ] Handle `assertType` in control flow analysis — if it throws, subsequent code has narrowed type
- [ ] Integrate with existing `NarrowingContext` in `crates/luanext-typechecker/src/visitors/narrowing.rs`

##### Phase 5: LSP & Tests

- [ ] LSP: hover on `assertType` shows intrinsic signature, completion suggests type arguments
- [ ] Unit tests: type checker validates intrinsic usage, rejects invalid type arguments
- [ ] Codegen tests: verify generated Lua for each type category (primitives, unions, optionals, classes)
- [ ] Integration tests: end-to-end compile + verify runtime behavior with correct and incorrect types
- [ ] Error message tests: verify descriptive messages for each failure mode

### Optimizer O2/O3 Passes

- [ ] Implement remaining O2 passes (function inlining, loop optimization, etc.)
- [ ] Implement O3 passes (aggressive inlining, devirtualization, etc.)

### Error Messages

- [ ] Improve type mismatch error messages with suggestions
- [ ] Add "did you mean?" suggestions for typos
- [ ] Better error recovery in parser

### Testing/Benchmarking Lua

- [ ] Consider a testing strategy for Lua code that results from the compilation process
- [ ] Consider a benchmarking strategy for Lua code that results from the compilation process
