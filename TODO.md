# LuaNext TODO

## Low Priority

### Language Features

#### Template Literal Patterns in Match Expressions

**Rationale:** The `match` expression currently supports literal, identifier, array, object, wildcard, and or-patterns. However, there's no way to destructure strings — a very common operation in parsers, URL routers, and data processing. Template literal patterns would allow extracting substrings directly in match arms, compiling to Lua's `string.match()` under the hood.

**Estimated Effort:** 2-3 weeks

**Syntax:**

```luanext
// Basic template pattern — destructure URL parts
const result = match url {
  `https://${host}/${path}` => processSecure(host, path),
  `http://${host}/${path}` => processInsecure(host, path),
  _ => error("Invalid URL")
}

// With guard clauses
const parsed = match input {
  `${name}:${age}` when tonumber(age) != nil => { name, age: tonumber(age) },
  _ => nil
}

// Mixed with other pattern types
const message = match value {
  `error: ${msg}` => handleError(msg),
  42 => "the answer",
  _ => "unknown"
}
```

**Current State:**

- ✅ **Phase 1-3 COMPLETE**: AST, Parser, Type Checker, and Codegen all implemented
- ✅ Template patterns parse from backtick syntax in match expressions
- ✅ Captures restricted to simple identifiers only
- ✅ Type checker validates string-only matching and infers `string` types for captures
- ✅ Codegen generates `string.match()` with delimiter-aware Lua patterns
- ✅ Project compiles successfully with all pattern match locations updated
- ⏳ **Phase 4-5 TODO**: LSP support and comprehensive test suite needed

**Benefits:**

- Enables string destructuring without manual `string.match()` calls
- Type-safe: captured variables are inferred as `string`
- Natural syntax for URL routing, log parsing, config parsing
- Compiles to efficient Lua patterns (no regex library needed)

**Compilation Target:**

```luanext
// Source:
match url {
  `https://${host}/${path}` => process(host, path),
  _ => nil
}

// Compiled Lua:
(function()
  local __match_value = url
  local __capture_1, __capture_2 = string.match(__match_value, "^https://([^/]+)/(.+)$")
  if __capture_1 ~= nil then
    local host = __capture_1
    local path = __capture_2
    return process(host, path)
  else
    return nil
  end
end)()
```

##### Phase 1: AST & Parser ✅ COMPLETE

- [x] Add `Template(TemplatePattern)` variant to `Pattern` enum in `crates/luanext-parser/src/ast/pattern.rs`
- [x] Define `TemplatePattern` struct with `parts: &'arena [TemplatePatternPart]` (String literal segments + Identifier captures)
- [x] Parse backtick-delimited patterns in match arms (reuse template literal lexer infrastructure)
- [x] Ensure `Pattern::span()` and `Pattern::node_id()` handle new variant
- [x] Validate captures are simple identifiers only (no expressions)
- [x] Detect and error on adjacent captures
- [x] Convert patterns without captures to `Pattern::Literal`

##### Phase 2: Type Checker ✅ COMPLETE

- [x] Validate template patterns only appear where the matched value is `string` type
- [x] Infer all captured variables as `string` type and register them in scope
- [x] Report error if template pattern used against non-string value
- [x] Handle template patterns in exhaustiveness checking (template patterns are non-exhaustive by nature)
- [x] Add `is_string_like_type()` helper for type validation
- [x] Update `extract_pattern_bindings_recursive()` for template patterns
- [x] Update `pattern_could_match()` and `narrow_type_by_pattern()`
- [x] Add template pattern handling in declaration phase

##### Phase 3: Codegen ✅ COMPLETE

- [x] Convert template pattern parts to Lua pattern string (literal text escaped, `${name}` becomes `(.+)` or `([^/]+)`)
- [x] Generate `string.match()` call in `generate_pattern_match()`
- [x] Generate local variable bindings from captures in `generate_pattern_bindings()`
- [x] Handle edge cases: empty captures, adjacent captures, special Lua pattern characters
- [x] Implement `generate_lua_pattern()` with delimiter-aware capture generation
- [x] Implement `escape_lua_pattern()` for Lua magic character escaping
- [x] Add template pattern handling to all pattern match locations
- [x] Add template pattern handling to optimizer passes

##### Phase 4: LSP Support

- [ ] Semantic tokens for captured variable names in template patterns
- [ ] Hover info showing inferred `string` type for captured variables
- [ ] Go-to-definition from captured variable usage to pattern binding site
- [ ] Completion for captured variables in match arm body

##### Phase 5: Tests

- [ ] Parser tests: basic template pattern, multiple captures, mixed with other patterns, error cases
- [ ] Type checker tests: string value matching, non-string error, guard clause interaction
- [ ] Codegen tests: verify generated Lua pattern syntax, variable binding, edge cases
- [ ] Integration tests: end-to-end compilation of template pattern match expressions

---

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

##### Phase 1: Intrinsic Recognition

- [ ] Register `assertType` as a compiler intrinsic in the type checker (not in stdlib)
- [ ] Validate: exactly one type argument required, exactly one value argument
- [ ] Return type is the type argument `T`
- [ ] Value argument accepts `unknown` or any supertype of `T`
- [ ] Report error if type argument is not a checkable type (e.g., generic type parameters)

##### Phase 2: Codegen for Primitive Types

- [ ] Recognize `assertType` calls in codegen expression handler
- [ ] Emit `type()` check for primitives: `string`, `number`, `boolean`, `nil`, `table`
- [ ] Generate descriptive error message: `"Type assertion failed: expected <T>, got " .. type(value)`
- [ ] Emit the value expression as the result (assertion returns the value)
- [ ] Handle `integer` type (Lua 5.3+: `math.type(x) == "integer"`, Lua 5.1/5.2: `type(x) == "number" and x % 1 == 0`)

##### Phase 3: Codegen for Complex Types

- [ ] **Union types**: `assertType<string | number>(x)` emits compound `type()` check with `and`
- [ ] **Optional types**: `assertType<string?>(x)` emits `x == nil or type(x) == "string"`
- [ ] **Class types**: `assertType<MyClass>(x)` emits table check + optional metatable/constructor check
- [ ] **Interface types**: structural check — verify required properties exist (optional: deep check behind a flag)
- [ ] **Literal types**: `assertType<"hello">(x)` emits `x == "hello"` value equality check

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
