# Lexer and Tokens

Tokenizes LuaNext source code into a stream of `Token` values, each carrying a `TokenKind` and a `Span`.

## Overview

The lexer (`crates/luanext-parser/src/lexer/`) converts raw source text into tokens. It handles keywords, identifiers, numbers, strings, template strings, operators, and delimiters. Identifier strings are deduplicated via a `StringInterner` that assigns each unique string a `StringId`.

## Token Structure

```rust
struct Token {
    kind: TokenKind,
    span: Span,
}
```

### Span

```rust
struct Span {
    start: u32,    // byte offset
    end: u32,      // byte offset
    line: u32,     // 0-based line number
    column: u32,   // 0-based column number
}
```

`[NOTE: DUPLICATE]` `Span` is defined in `luanext-parser` and re-used by `luanext-sourcemap`. The sourcemap crate has its own span-like structures for position translation.

## Keywords (64 total)

Keyword lookup uses length-based bucketing for O(1) length check before string comparison.

| Length | Keywords |
| ------ | -------- |
| 2 | `do`, `if`, `in`, `or`, `as`, `is` |
| 3 | `end`, `for`, `and`, `not`, `nil`, `get`, `set`, `new`, `try` |
| 4 | `then`, `else`, `type`, `enum`, `from`, `when`, `true`, `goto` |
| 5 | `const`, `local`, `while`, `break`, `until`, `false`, `match`, `class`, `super`, `keyof`, `infer`, `final`, `throw`, `catch` |
| 6 | `return`, `global`, `elseif`, `repeat`, `import`, `export`, `public`, `static`, `throws`, `typeof` |
| 7 | `private`, `extends`, `declare`, `finally`, `rethrow` |
| 8 | `function`, `continue`, `abstract`, `operator`, `readonly`, `override` |
| 9 | `interface`, `protected`, `namespace` |
| 10 | `implements`, `instanceof` |
| 11 | `constructor` |

`[NOTE: BEHAVIOR]` Keywords can be used as method names, member names, and table property keys (e.g., `obj.get`, `{ type = 5 }`). The parser uses `parse_identifier_or_keyword()` in these contexts. Keywords cannot be used as variable names — `parse_identifier()` rejects them.

`[NOTE: BEHAVIOR]` `new`, `operator`, `throw`, `try`, `catch`, `finally`, `rethrow`, `throws` are keywords but not listed in `is_keyword()` — they have dedicated `TokenKind` variants but are handled separately in parsing contexts.

## Operators

### Arithmetic

`+`, `-`, `*`, `/`, `%`, `^`, `//` (integer divide)

### Comparison

`==`, `!=`, `<`, `<=`, `>`, `>=`

`[NOTE: BEHAVIOR]` `~=` is also recognized for Lua-style not-equal, but `!=` is the primary form.

### Logical

`and`, `or`, `not`

### Bitwise

`&`, `|`, `~`, `<<`, `>>` (plus `~` as unary bitwise NOT)

### String

`..` (concatenate)

### Assignment (13 operators)

`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `..=`, `&=`, `|=`, `//=`, `<<=`, `>>=`

### Special Operators

| Token | Symbol | Purpose |
| ----- | ------ | ------- |
| `Arrow` | `->` | Return type annotation |
| `FatArrow` | `=>` | Arrow function body |
| `PipeOp` | `\|>` | Pipe operator |
| `Question` | `?` | Nullable type / ternary |
| `QuestionQuestion` | `??` | Null coalescing |
| `QuestionDot` | `?.` | Optional chaining |
| `ColonColon` | `::` | Method call (Lua-style) |
| `DotDotDot` | `...` | Spread / rest / variadic |
| `Bang` | `!` | Non-null assertion |
| `BangBang` | `!!` | Double-bang assertion |
| `At` | `@` | Decorator prefix |

### Delimiters

`(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`

## Identifiers

Identifiers are stored as `StringId` values, not raw strings. The `TokenKind::Identifier(StringId)` variant holds the interned ID.

## Literals

| Kind | Token Representation |
| ---- | -------------------- |
| Number | `TokenKind::Number(String)` — raw text, parsed later |
| String | `TokenKind::String(String)` — content without quotes |
| Template | `TokenKind::TemplateString(Vec<TemplatePart>)` |

### Template Strings

Template literals use backtick syntax: `` `hello ${name}` ``

```rust
enum TemplatePart {
    String(String),          // literal text segments
    Expression(Vec<Token>),  // interpolated expression tokens
}
```

The lexer tokenizes template strings into alternating string/expression parts. Expression parts contain pre-lexed token vectors.

## String Interner

**File**: `crates/luanext-parser/src/string_interner.rs`

```rust
struct StringInterner {
    rodeo: Arc<ThreadedRodeo>,  // lasso crate
}

struct StringId(lasso::Spur);  // Copy, Eq, Hash, Serialize, Deserialize
```

### Key Operations

| Method | Behavior |
| ------ | -------- |
| `intern(s)` | Get-or-insert, returns `StringId` |
| `resolve(id)` | ID → `String` (panics if invalid) |
| `try_resolve(id)` | ID → `Option<String>` |
| `with_resolved(id, f)` | Zero-alloc callback with `&str` |
| `to_strings()` | Export all strings for serialization |
| `from_strings(vec)` | Reconstruct interner from exported strings |

### CommonIdentifiers

Pre-registered identifiers for frequently used strings (`nil`, `true`, `false`, `and`, `or`, `not`, `function`, `local`, `const`, `return`, `if`, `elseif`, `else`, `then`, `end`, `while`, `do`, `for`, `in`, `break`, `continue`, `repeat`, `until`, `key`).

Created via `StringInterner::new_with_common_identifiers()`.

### Thread Safety

`StringInterner` wraps `Arc<ThreadedRodeo>` — fully thread-safe for concurrent `intern()` calls across rayon workers during parallel parsing. `StringId` is `Copy + Send + Sync`.

### Serialization

`StringId` derives `Serialize`/`Deserialize` for cache storage. The interner itself is serialized via `to_strings()` / `from_strings()` for the incremental compilation cache.

## Cross-References

- [Parser](parser.md) — consumes token stream
- [AST](ast.md) — uses `StringId` for identifier nodes
- [Incremental Cache](../compiler/incremental-cache.md) — serializes interner state
