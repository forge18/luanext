# Parser

Recursive descent parser that transforms a token stream into an arena-allocated AST.

## Overview

The parser (`crates/luanext-parser/src/parser/`) consumes a `Vec<Token>` produced by the [lexer](lexer-and-tokens.md) and emits a `Program<'arena>` containing arena-allocated [AST](ast.md) nodes. It is split into four submodule traits that are all implemented on a single `Parser<'a, 'arena>` struct:

| Submodule | Trait | Entry point | Responsibility |
| --------- | ----- | ----------- | -------------- |
| `statement.rs` | `StatementParser` | `parse_statement()` | All statement forms (variable, function, class, control flow, etc.) |
| `expression.rs` | `ExpressionParser` | `parse_expression()` | Expressions with precedence climbing |
| `pattern.rs` | `PatternParser` | `parse_pattern()` | Match/destructuring patterns |
| `mod.rs` | -- | `parse()` | Top-level loop, token stream management, error recovery, incremental parsing |
| `types.rs` | `TypeParser` | `parse_type()` | Type annotations (union, intersection, conditional, mapped, etc.) |

Each trait has a single public method that the other submodules call via their trait bounds. Helper methods are `pub(super)` so they can be shared across submodules.

### Auxiliary modules

| File | Purpose |
| ---- | ------- |
| `di.rs` | Generic DI container (`DiContainer`) with transient and singleton lifetimes. Used for parser setup and configuration. Not specific to the parser itself. |
| `arena_test.rs` | Unit tests for arena allocation behavior |

## Arena Allocation

All AST nodes have lifetime `'arena`, tied to a `bumpalo::Bump` allocator passed at construction.

```rust
pub struct Parser<'a, 'arena> {
    tokens: Vec<Token>,
    position: usize,
    diagnostic_handler: Arc<dyn DiagnosticHandler>,
    interner: &'a StringInterner,
    common: &'a CommonIdentifiers,
    arena: &'arena Bump,
    has_namespace: bool,
    is_first_statement: bool,
}
```

### Allocation helpers

The parser provides two inline helpers that delegate to the arena:

```rust
fn alloc<T>(&self, value: T) -> &'arena T           // single value
fn alloc_vec<T>(&self, vec: Vec<T>) -> &'arena [T]  // Vec -> arena slice
```

Both rely on `Bump`'s interior mutability (`&Bump` suffices, no `&mut Bump` required). The `alloc` calls are safe because the arena outlives the parser.

### Program vs MutableProgram

| Type | Statements field | Purpose |
| ---- | ---------------- | ------- |
| `Program<'arena>` | `&'arena [Statement<'arena>]` | Immutable arena-allocated output from the parser |
| `MutableProgram<'arena>` | `Vec<Statement<'arena>>` | Defined in `luanext-core`, used by the optimizer for in-place transforms |

`MutableProgram::from_program()` copies the arena slice into a `Vec` for mutation. This pattern parallels rustc's HIR (immutable) / MIR (mutable) separation.

### Ident and StringId

```rust
pub type Ident = Spanned<StringId>;

pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
```

All identifier strings are interned via `StringInterner`. The parser holds references to `&'a StringInterner` and `&'a CommonIdentifiers` for fast access to frequently-used identifiers.

## Parser Architecture

### Construction

```rust
Parser::new(
    tokens: Vec<Token>,
    diagnostic_handler: Arc<dyn DiagnosticHandler>,
    interner: &'a StringInterner,
    common: &'a CommonIdentifiers,
    arena: &'arena Bump,
) -> Self
```

### Top-level parse loop

`parse()` iterates the token stream, calling `parse_statement()` for each statement. Semicolons between statements are consumed and discarded (Lua compatibility). On parse failure, `report_parser_error()` emits a diagnostic and `synchronize()` skips ahead to the next statement boundary.

```lua
while !is_at_end() {
    skip semicolons
    match parse_statement() {
        Ok(stmt) -> push to statements
        Err(e) -> report + synchronize
    }
}
allocate statements slice in arena -> Program
```

### Token stream primitives

| Method | Description |
| ------ | ----------- |
| `current()` | Returns the token at `position` (falls back to the last token on overflow) |
| `is_at_end()` | True when current token is `Eof` |
| `advance()` | Moves position forward by 1, returns the consumed token |
| `check(&TokenKind)` | Checks discriminant equality without advancing |
| `nth_token_kind(n)` | Lookahead by `n` tokens from current position |
| `match_token(&[TokenKind])` | If current matches any of the kinds, advances and returns true |
| `consume(kind, msg)` | Like `match_token` for a single kind, but returns `Err(ParserError)` on mismatch |

All of these except `nth_token_kind` are `#[inline(always)]`.

### Namespace and first-statement tracking

The parser tracks two boolean flags:

- `has_namespace` -- set to true when a `namespace` statement is parsed. Used to enforce that at most one namespace declaration exists per file.
- `is_first_statement` -- set to false after the first statement is parsed. Used to enforce that `namespace` declarations must be the first statement.

## Statement Parsing

`parse_statement()` dispatches on the current token kind:

| Token(s) | Statement type | Parser method |
| -------- | -------------- | ------------- |
| `const`, `local`, `global` | `Variable` | `parse_variable_declaration()` |
| `function` | `Function` | `parse_function_declaration()` |
| `if` | `If` | `parse_if_statement()` |
| `while` | `While` | `parse_while_statement()` |
| `for` | `For` (Numeric or Generic) | `parse_for_statement()` |
| `repeat` | `Repeat` | `parse_repeat_statement()` |
| `return` | `Return` | `parse_return_statement()` |
| `break` | `Break` | direct advance |
| `continue` | `Continue` | direct advance |
| `interface` | `Interface` | `parse_interface_declaration()` |
| `type` | `TypeAlias` | `parse_type_alias_declaration()` |
| `enum` | `Enum` | `parse_enum_declaration()` |
| `import` | `Import` | `parse_import_declaration()` |
| `export` | `Export` | `parse_export_declaration()` |
| `abstract`, `final`, `class` | `Class` | `parse_class_declaration()` |
| `@` (decorator) | `Class` | `parse_class_declaration()` |
| `declare` | `DeclareFunction`, `DeclareNamespace`, `DeclareType`, `DeclareInterface`, `DeclareConst` | `parse_declare_statement()` |
| `throw` | `Throw` | `parse_throw_statement()` |
| `try` | `Try` | `parse_try_statement()` |
| `rethrow` | `Rethrow` | `parse_rethrow_statement()` |
| `namespace` | `Namespace` | `parse_namespace_declaration()` |
| `goto` | `Goto` | `parse_goto_statement()` |
| `::` | `Label` | `parse_label_statement()` |
| `{` or `[` (when destructuring) | `Variable` (Global kind) | `parse_destructuring_assignment()` |
| *(other)* | `Expression` or `MultiAssignment` | Falls through to expression parsing |

### Implicit global detection

Before the main dispatch, the parser checks `is_implicit_global_declaration()`. This handles the syntax `x: number = 42` (implicit global declaration) and must be distinguished from `x :: T` (type assertion / method call).

The lookahead scans from the token after the colon, tracking bracket/paren/brace nesting depth, and looks for an `=` at depth 0 within 50 tokens. Statement boundary tokens at depth 0 (`end`, `else`, `then`, etc.) terminate the search with a negative result.

### Multi-assignment

When expression parsing encounters a comma after the first expression, the parser checks for multi-assignment syntax: `a, b = swap(1, 2)`. Targets are parsed with `parse_conditional()` (not `parse_expression()`) to avoid consuming the `=` as part of an assignment expression.

### Block parsing

`parse_block()` collects statements until a block terminator is reached: `end`, `else`, `elseif`, `until`, or `}`. Semicolons are skipped between statements.

## Block Syntax Rules

LuaNext supports two block styles. For any given block, use EITHER braces OR keyword delimiters -- never mix them.

| Block style | Syntax |
| ----------- | ------ |
| Keyword (Lua-style) | `do ... end`, `then ... end`, `then ... else ... end` |
| Brace (TS-style) | `{ ... }` |

`[NOTE: BEHAVIOR]` For-loops always use `do ... end` style. If-statements and while-loops can use either style. Mixing styles within a single block (e.g., `for i in x { ... end`) is invalid and causes a silent parse failure -- the parser's error recovery drops the malformed statement with no obvious error about the block style mismatch.

`[NOTE: GOTCHA]` This has caused silent failures in tests multiple times. The parser error recovery drops the malformed statement, leading to misleading "bugs" where statements seem to disappear.

## Expression Parsing

Expression parsing uses a precedence climbing approach, where each precedence level is a separate method that calls the next-tighter level. This is effectively Pratt parsing laid out as recursive descent.

### Precedence table (lowest to highest)

| Level | Method | Operators | Associativity |
| ----- | ------ | --------- | ------------- |
| 1 | `parse_assignment` | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `..=`, `&=`, `\|=`, `//=`, `<<=`, `>>=` | Right |
| 2 | `parse_conditional` | `? :` (ternary) | Right |
| 3 | `parse_logical_or` | `or` | Left |
| 4 | `parse_null_coalesce` | `??` | Left |
| 5 | `parse_logical_and` | `and` | Left |
| 6 | `parse_bitwise_or` | `\|` | Left |
| 7 | `parse_bitwise_xor` | `~` (binary) | Left |
| 8 | `parse_bitwise_and` | `&` | Left |
| 9 | `parse_equality` | `==`, `!=`, `~=` | Left |
| 10 | `parse_comparison` | `<`, `<=`, `>`, `>=` | Left |
| 11 | `parse_concatenation` | `..` | Left |
| 12 | `parse_shift` | `<<`, `>>` | Left |
| 13 | `parse_additive` | `+`, `-` | Left |
| 14 | `parse_multiplicative` | `*`, `/`, `//`, `%` | Left |
| 15 | `parse_power` | `^` | Right |
| 16 | `parse_unary` | `-`, `not`, `!`, `#`, `~` (unary), `new` | Right (prefix) |
| 17 | `parse_postfix` | `.`, `[]`, `()`, `::`, `\|>`, `?.`, `!!`, `<T>()` | Left (postfix) |
| 18 | `parse_primary` | Literals, identifiers, `(expr)`, `{...}`, `[...]`, `function`, `match`, template strings, `super` | -- |

`[NOTE: BEHAVIOR]` Arrow functions (`x => x + 1`, `(x, y) => x + y`) are tried first at the assignment level via `try_parse_arrow_function()` with backtracking. If the arrow function parse fails, the parser resets to the checkpoint and falls through to conditional parsing.

### Postfix operators

The postfix parsing loop in `parse_postfix()` handles all postfix operations in a single loop:

| Token | Expression kind | Notes |
| ----- | --------------- | ----- |
| `.` | `Member` | Uses `parse_identifier_or_keyword()` for the member name |
| `[expr]` | `Index` | |
| `(args)` | `Call` | |
| `<Types>(args)` | `Call` with type args | Uses backtracking -- if `<` doesn't lead to `>(`, treated as comparison |
| `::` | `MethodCall` | Uses `parse_identifier_or_keyword()` for the method name |
| `\|>` | `Pipe` | |
| `?.` | `OptionalMember`, `OptionalIndex`, `OptionalCall`, or `OptionalMethodCall` | Dispatches on next token after `?.` |
| `!!` | `ErrorChain` | |

### Generic call ambiguity

When the parser sees `<` after an expression, it tentatively tries to parse type arguments. If the parse succeeds and the next token after `>` is `(`, it commits to a generic call (`foo<number>(42)`). Otherwise, it backtracks and leaves `<` for the comparison level to handle.

`[NOTE: BEHAVIOR]` The `>>` token is handled specially for nested generics (`Map<string, Array<number>>`). When `consume_closing_angle_bracket()` encounters `>>`, it mutates the token in-place to a single `>` without advancing, so the outer generic context can consume the remaining `>`.

### Primary expressions

| Token | Expression kind |
| ----- | --------------- |
| `nil` | `Literal(Nil)` |
| `true`, `false` | `Literal(Boolean)` |
| `Number(s)` | `Literal(Number)` -- supports decimal, hex (`0x`/`0X`), binary (`0b`/`0B`) |
| `String(s)` | `Literal(String)` |
| `Identifier` | `Identifier` |
| `type` | `Identifier` -- special case: `type` keyword treated as identifier in expression context (for Lua stdlib `type()` function) |
| `(expr)` | `Parenthesized` -- handles multiple nested parens |
| `{...}` | `Object` -- table/object literal |
| `[...]` | `Array` |
| `function` | `Function` (anonymous function expression) |
| `match` | `Match` |
| `TemplateString` | `Template` -- template literal with interpolation |
| `super` | `SuperKeyword` |

`[NOTE: BEHAVIOR]` The `type` keyword is both a keyword and a Lua standard library function. In expression context (e.g., `type(value)`), the parser treats it as an identifier by interning the string "type". In statement context, it triggers `parse_type_alias_declaration()`.

### Object/table literals

Object parsing (`parse_object_or_table()`) supports several property forms:

| Form | AST representation |
| ---- | ------------------ |
| `{ key = value }` or `{ key: value }` | `ObjectProperty::Property` |
| `{ x }` | Shorthand -- equivalent to `{ x = x }` |
| `{ [expr] = value }` | `ObjectProperty::Computed` |
| `{ ...obj }` | `ObjectProperty::Spread` |
| `{ 1, 2, 3 }` | Positional -- auto-incremented integer keys via `Computed` |

`[NOTE: BEHAVIOR]` Keywords can be used as property keys when followed by `=` or `:`. The parser checks `is_keyword()` with lookahead to distinguish `{ get = 42 }` (property) from `{ get }` (shorthand).

## Type Parsing

Type parsing follows a similar precedence structure:

| Level | Method | Type forms |
| ----- | ------ | ---------- |
| 1 | `parse_type` | Type predicates (`x is T`) with backtracking |
| 2 | `parse_union_type` | `A \| B \| C` |
| 3 | `parse_intersection_type` | `A & B & C` |
| 4 | `parse_conditional_type` | `T extends U ? A : B` |
| 5 | `parse_postfix_type` | `T?` (nullable), `T[]` (array), `keyof T`, `typeof T` |
| 6 | `parse_primary_type` | Primitive types, named types, object types, tuple types, function types, mapped types, template literal types |

## Pattern Parsing

Pattern parsing (`parse_pattern()`) starts with `parse_or_pattern()`:

| Pattern form | AST representation |
| ------------ | ------------------ |
| `_` | `Pattern::Wildcard` |
| `name` | `Pattern::Identifier` |
| `true`, `false` | `Pattern::Literal(Boolean)` |
| Number, String | `Pattern::Literal` |
| `{ ... }` | Object destructuring pattern |
| `[ ... ]` | Array destructuring pattern |
| `A \| B` | `Pattern::Or` |
| Keywords | Treated as identifiers (for function type parameters) |

## Identifier Handling

Two methods exist for parsing identifiers:

### `parse_identifier()`

Accepts **only** `TokenKind::Identifier`. Rejects all keywords. Used for:

- Variable names in declarations
- Function names
- Type names
- Label names
- Goto targets

### `parse_identifier_or_keyword()`

Accepts both `TokenKind::Identifier` and any keyword token. Keywords are interned via `kind.to_keyword_str()`. Used for:

- Member names after `.` (e.g., `obj.get`)
- Method names after `::` (e.g., `b::get()`)
- Table/object property keys (e.g., `{ get = 42 }`)
- Interface member names
- Optional chaining member/method names

`[NOTE: BEHAVIOR]` This distinction is critical. Using `parse_identifier()` where `parse_identifier_or_keyword()` is needed causes silent drops -- for example, `b::get()` previously failed silently because `get` is a keyword and `parse_identifier()` rejected it. The fix was to use `parse_identifier_or_keyword()` in all member/method name positions in `parse_postfix()`.

### `parse_interface_member_name()`

A specialized variant for interface member names that accepts identifiers and keywords. Functionally equivalent to `parse_identifier_or_keyword()` but defined separately in the interface parsing context.

## Error Recovery

### Synchronization

On parse error, the parser calls `synchronize()` which advances the token stream until it reaches a statement boundary token. The synchronization targets are:

**Statement starters** (resume parsing here):
`function`, `local`, `const`, `global`, `if`, `while`, `for`, `repeat`, `return`, `break`, `continue`, `interface`, `type`, `enum`, `class`, `import`, `export`, `declare`, `namespace`, `;`

**Block terminators** (stop advancing, let the block parser handle):
`end`, `elseif`, `else`, `until`

### Error reporting

Errors are classified by `classify_error_code()` which pattern-matches on the error message text to assign diagnostic error codes:

| Code | Condition |
| ---- | --------- |
| `E2020` | "break" + "outside" |
| `E2021` | "continue" + "outside" |
| `E2010` | Contains "end" |
| `E2011` | Contains "then" |
| `E2012` | Contains "do" + ("while" or "for") |
| `E2003` | "identifier" + "Expected" |
| `E2004` | "expression" + "Expected" |
| `E2002` | "Expected" |
| `E2001` | "Unexpected" (default) |

### Contextual suggestions

`suggest_for_expected()` provides actionable fix suggestions for common missing tokens:

| Missing token | Suggestion |
| ------------- | ---------- |
| `end` | "Add 'end' to close this block" |
| `then` | "Add 'then' after the condition" |
| `do` | "Add 'do' after the condition" |
| `until` | "Add 'until' followed by a condition" |
| `)` | "Add ')' to close the opening '('" |
| `]` | "Add ']' to close the opening '['" |
| `}` | "Add '}' to close the opening '{'" |

The `consume()` method also detects `=` vs `==` typos and suggests the correct operator.

### ParserError structure

```rust
pub struct ParserError {
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}
```

`[NOTE: BEHAVIOR]` Error recovery means the parser continues after errors, dropping malformed statements. This is by design for IDE use -- a file with errors still produces a partial AST. However, it can mask block syntax violations where the parser silently drops an entire statement.

## Implicit Global Detection

The `is_implicit_global_declaration()` method distinguishes three forms that start with `Identifier :`:

| Pattern | Meaning | How detected |
| ------- | ------- | ------------ |
| `x: number = 42` | Implicit global declaration | Colon not followed by colon, `=` found at depth 0 |
| `x :: T` | Type assertion / method call | Second token after ident is also `:` |
| `x: foo()` | Not a declaration | No `=` found before statement boundary |

The lookahead algorithm:

1. Verify `Identifier` at position 0, `:` at position 1, NOT `:` at position 2
2. Scan from position 2 forward (up to 50 tokens)
3. Track nesting depth across `< > ( ) { } [ ]`
4. Return `true` if `=` found at depth 0
5. Return `false` if statement boundary token found at depth 0 or EOF reached

### Destructuring assignment detection

`is_destructuring_assignment()` handles `{x, y} = obj` and `[a, b] = arr`. It scans forward from the opening bracket to the matching close at depth 0, then checks if `=` immediately follows.

## Incremental Parsing

`[NOTE: FEATURE]` Gated behind the `incremental-parsing` feature flag.

`parse_incremental()` provides three paths for efficient re-parsing after edits:

| Path | Condition | Cost |
| ---- | --------- | ---- |
| **First parse** | No previous tree | Full parse, builds initial `IncrementalParseTree` |
| **No edits** | Empty edits, source hash matches | O(n) pointer copy, no parsing |
| **Partial edits** | Some statements overlap edits | Re-lex and re-parse only dirty regions |

Clean vs dirty classification uses `is_statement_clean()` which checks byte-range overlap against edit ranges. If all statements are dirty, falls back to full parse.

The incremental system maintains multiple arenas via `Rc<Bump>` and uses `unsafe transmute` for lifetime casting between `'arena` and `'static`. Garbage collection triggers at >3 arenas or every 10 versions, consolidating all statements into a fresh arena.

For full details, see [Incremental Parsing](../compiler/incremental-parsing.md).

### Region parsing

`parse_region()` is used by incremental parsing to re-parse a byte range of the source. It creates a temporary lexer for the region, appends an `Eof` token, swaps the parser's token stream, parses statements until done, then restores the original token stream.

## Cross-References

- [Lexer and Tokens](lexer-and-tokens.md) -- token stream input, keyword list, operator tokens
- [AST](ast.md) -- output node types (`Statement`, `Expression`, `Type`, `Pattern`)
- [Incremental Parsing](../compiler/incremental-parsing.md) -- multi-arena system, GC, consolidation
- `crates/luanext-parser/src/parser/mod.rs` -- main parser struct and top-level parse loop
- `crates/luanext-parser/src/parser/statement.rs` -- statement parsing, identifier helpers
- `crates/luanext-parser/src/parser/expression.rs` -- expression parsing with precedence
- `crates/luanext-parser/src/parser/types.rs` -- type annotation parsing
- `crates/luanext-parser/src/parser/pattern.rs` -- match/destructuring pattern parsing
- `crates/luanext-core/src/lib.rs` -- `MutableProgram` definition
