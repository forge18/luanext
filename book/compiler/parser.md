# Parser Architecture

The LuaNext parser transforms token streams into Abstract Syntax Trees (ASTs) using recursive descent parsing with Pratt parsing for expression precedence. This document explains the parser's architecture, design patterns, and how to extend it.

## Table of Contents

1. [Lexer Architecture](#lexer-architecture)
2. [Recursive Descent Parsing](#recursive-descent-parsing)
3. [Operator Precedence](#operator-precedence)
4. [Error Recovery](#error-recovery)
5. [Span Tracking](#span-tracking)
6. [Adding New Syntax](#adding-new-syntax)
7. [AST Node Design Patterns](#ast-node-design-patterns)
8. [Testing Parser Changes](#testing-parser-changes)

---

## 1. Lexer Architecture

**Location:** `/crates/luanext-parser/src/lexer/`

### Token Stream Generation

The lexer (`Lexer`) converts raw source code into a vector of tokens:

```rust
pub struct Lexer<'a> {
    source: Vec<char>,          // Pre-converted character array for O(1) indexing
    position: u32,              // Current byte offset
    line: u32,                  // Current line (1-indexed)
    column: u32,                // Current column (1-indexed)
    diagnostic_handler: Arc<dyn DiagnosticHandler>,
    interner: &'a StringInterner,
}
```

**Key Design Decisions:**

- **Character Array:** Source is converted to `Vec<char>` upfront for efficient random access and Unicode handling
- **Compact Positions:** Uses `u32` instead of `usize` (supports files up to 4GB)
- **Pre-allocation:** Estimates token count based on source length to minimize reallocations

### String Interning

The lexer uses a **thread-safe string interner** to deduplicate identifier strings:

```rust
pub struct StringInterner {
    rodeo: Arc<ThreadedRodeo>,  // Thread-safe interner from lasso crate
}

pub struct StringId(lasso::Spur);  // Compact identifier handle
```

**Why String Interning?**

1. **Memory Efficiency:** "myVariable" appears once in memory regardless of how many times it's used
2. **Fast Comparisons:** Comparing `StringId` (32-bit integer) is faster than string comparisons
3. **Thread-Safe:** Multiple threads can intern strings concurrently
4. **Serializable:** Can export/import string tables for caching

**Common Identifiers:**

Frequently used identifiers are pre-registered for O(1) lookup:

```rust
pub struct CommonIdentifiers {
    pub nil: StringId,
    pub true_: StringId,
    pub false_: StringId,
    pub function: StringId,
    // ... 20+ common keywords
}
```

### Lexeme Classification

**Location:** `/crates/luanext-parser/src/lexer/lexeme.rs`

Tokens are classified by length-bucketed keyword matching for optimal performance:

```rust
pub fn from_keyword(s: &str) -> Option<TokenKind> {
    match s.len() {
        2 => match s {
            "do" => Some(TokenKind::Do),
            "if" => Some(TokenKind::If),
            // ...
        },
        3 => match s { /* ... */ },
        // ... up to length 11 for "constructor"
        _ => None,
    }
}
```

This achieves **O(1)** length check + small constant-time string comparison per bucket.

### Performance Optimizations

1. **String Pre-scanning:**
   ```rust
   // First pass: find string end and check for escapes
   while end_pos < len && source[end_pos] != quote {
       if source[end_pos] == '\\' {
           has_escapes = true;
           end_pos += 1; // Skip next char
       }
       end_pos += 1;
   }

   // Second pass: build string with precise allocation
   if !has_escapes {
       // Fast path: direct slice conversion
   } else {
       // Slow path: process escape sequences
   }
   ```

2. **Template Literal Parsing:**
   - Pre-allocates for 4 template parts and 32-char string segments
   - Tracks brace depth for nested interpolations: `` `${foo({ bar: baz })}` ``

3. **Comment Handling:**
   - Single-line: `-- comment`
   - Multi-line: `--[[ comment ]]--`
   - Unterminated multi-line comments report diagnostics but don't halt lexing

### Number Literals

Supports multiple formats:
- **Decimal:** `123`, `45.67`, `1e10`, `2.5e-3`
- **Hexadecimal:** `0xFF`, `0x1A`
- **Binary:** `0b1010`, `0b1111`

---

## 2. Recursive Descent Parsing

**Location:** `/crates/luanext-parser/src/parser/mod.rs`

### Parser Structure

```rust
pub struct Parser<'a, 'arena> {
    tokens: Vec<Token>,                              // Token stream
    position: usize,                                 // Current token index
    diagnostic_handler: Arc<dyn DiagnosticHandler>,  // Error reporting
    interner: &'a StringInterner,                    // String deduplication
    common: &'a CommonIdentifiers,                   // Pre-registered identifiers
    arena: &'arena Bump,                             // Arena allocator for AST nodes
    has_namespace: bool,                             // Namespace tracking
    is_first_statement: bool,                        // First statement check
}
```

### Modular Parsing Traits

The parser is split across multiple modules using **traits for separation of concerns:**

```rust
pub trait ExpressionParser<'arena> {
    fn parse_expression(&mut self) -> Result<Expression<'arena>, ParserError>;
}

pub trait StatementParser<'arena> {
    fn parse_statement(&mut self) -> Result<Statement<'arena>, ParserError>;
}

pub trait TypeParser<'arena> {
    fn parse_type(&mut self) -> Result<Type<'arena>, ParserError>;
}

pub trait PatternParser<'arena> {
    fn parse_pattern(&mut self) -> Result<Pattern<'arena>, ParserError>;
}
```

**Implementation Files:**
- `parser/expression.rs` - Expression parsing (400+ lines)
- `parser/statement.rs` - Statement parsing (600+ lines)
- `parser/types.rs` - Type annotation parsing
- `parser/pattern.rs` - Pattern matching syntax

### Token Consumption API

```rust
// Check if current token matches a kind
fn check(&self, kind: &TokenKind) -> bool

// Advance if current token matches any kind
fn match_token(&mut self, kinds: &[TokenKind]) -> bool

// Consume token or error
fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ParserError>

// Lookahead without consuming
fn nth_token_kind(&self, n: usize) -> Option<&TokenKind>
```

### Arena Allocation Pattern

All AST nodes use **arena allocation** to eliminate heap fragmentation and improve cache locality:

```rust
// Single value allocation
fn alloc<T>(&self, value: T) -> &'arena T {
    self.arena.alloc(value)
}

// Slice allocation from Vec
fn alloc_vec<T>(&self, vec: Vec<T>) -> &'arena [T] {
    self.arena.alloc_slice_fill_iter(vec.into_iter())
}
```

**Why Arena Allocation?**

1. **Fast Allocation:** Bump pointer allocation is ~10x faster than malloc
2. **No Individual Frees:** Entire AST freed at once when arena drops
3. **Cache Friendly:** Related nodes are allocated sequentially in memory
4. **Lifetime Management:** Rust's lifetime checker ensures no dangling references

---

## 3. Operator Precedence

**Location:** `/crates/luanext-parser/src/parser/expression.rs`

### Pratt Parsing (Recursive Descent with Precedence)

LuaNext uses **recursive descent** where each precedence level is a separate function. Higher precedence operators are parsed deeper in the call stack.

### Precedence Hierarchy (Lowest to Highest)

```
Assignment          =, +=, -=, *=, /=, %=, ^=, ..=, &=, |=, //=, <<=, >>=
  ↓
Conditional         ?:
  ↓
Logical OR          or
  ↓
Null Coalesce       ??
  ↓
Logical AND         and
  ↓
Bitwise OR          |
  ↓
Bitwise XOR         ~
  ↓
Bitwise AND         &
  ↓
Equality            ==, !=, ~=
  ↓
Comparison          <, <=, >, >=, instanceof
  ↓
Concatenation       ..
  ↓
Shift               <<, >>
  ↓
Additive            +, -
  ↓
Multiplicative      *, /, //, %
  ↓
Power               ^  (RIGHT ASSOCIATIVE)
  ↓
Unary               not, -, #, ~, new
  ↓
Postfix             ., [], (), <TypeArgs>(), ?., ?[], ?()
  ↓
Primary             identifiers, literals, self, super, templates, ()
```

### Implementation Pattern

Each precedence level follows this pattern:

```rust
fn parse_<level>(&mut self) -> Result<Expression<'arena>, ParserError> {
    let mut expr = self.parse_<next_higher_level>()?;

    while let Some(op) = self.match_<operator_type>() {
        let right = self.parse_<next_higher_level>()?;
        let span = expr.span.combine(&right.span);
        expr = Expression {
            kind: ExpressionKind::Binary(op, self.alloc(expr), self.alloc(right)),
            span,
            ..Default::default()
        };
    }

    Ok(expr)
}
```

### Right Associativity

Power operator `^` is **right associative** (unlike most binary operators):

```rust
fn parse_power(&mut self) -> Result<Expression<'arena>, ParserError> {
    let expr = self.parse_unary()?;

    if self.match_token(&[TokenKind::Caret]) {
        let right = self.parse_power()?;  // Recursive call (not loop)
        return Ok(/* binary power expression */);
    }

    Ok(expr)
}
```

This ensures `2^3^4` parses as `2^(3^4)` = 2^81, not `(2^3)^4` = 8^4.

### Generic Type Arguments Ambiguity

The `<` token is ambiguous: less-than operator vs. generic type argument start.

**Solution: Backtracking**

```rust
TokenKind::LessThan => {
    let checkpoint = self.position;
    self.advance();

    if let Ok(type_args) = self.parse_type_arguments() {
        if self.check(&TokenKind::GreaterThan) {
            self.advance();
            if self.check(&TokenKind::LeftParen) {
                // It's a generic call: foo<T>(args)
                // ... parse arguments ...
            } else {
                // Not followed by '(' - backtrack
                self.position = checkpoint;
                break;
            }
        } else {
            // Type args didn't parse correctly - backtrack
            self.position = checkpoint;
            break;
        }
    } else {
        // Failed to parse type args - backtrack
        self.position = checkpoint;
        break;
    }
}
```

### `>>` Token Splitting

The `>>` token must be split when closing nested generics:

```rust
fn consume_closing_angle_bracket(&mut self) -> Result<(), ParserError> {
    if self.check(&TokenKind::GreaterThan) {
        self.advance();
        Ok(())
    } else if self.check(&TokenKind::GreaterGreater) {
        // Split '>>' into two '>' by mutating token in-place
        self.tokens[self.position].kind = TokenKind::GreaterThan;
        Ok(())  // Don't advance - outer context will consume remaining '>'
    } else {
        Err(ParserError { /* ... */ })
    }
}
```

This allows parsing `Map<string, Array<number>>` without special lookahead.

---

## 4. Error Recovery

### Synchronization Strategy

When a parse error occurs, the parser **synchronizes** to the next statement boundary:

```rust
fn synchronize(&mut self) {
    self.advance();

    while !self.is_at_end() {
        match &self.current().kind {
            // Statement keywords
            TokenKind::Function | TokenKind::Local | TokenKind::Const |
            TokenKind::If | TokenKind::While | TokenKind::For |
            TokenKind::Return | TokenKind::Break | TokenKind::Continue |
            TokenKind::Class | TokenKind::Interface | TokenKind::Type |
            TokenKind::Enum | TokenKind::Import | TokenKind::Export |
            TokenKind::Declare | TokenKind::Namespace |
            // Delimiters
            TokenKind::Semicolon | TokenKind::End |
            TokenKind::Elseif | TokenKind::Else | TokenKind::Until => return,
            _ => {}
        }
        self.advance();
    }
}
```

**Why This Works:**

1. **Statement Boundaries:** Keywords like `function`, `local`, `if` always start new statements
2. **Block Delimiters:** `end`, `elseif`, `else` close blocks and start new contexts
3. **Skip Garbage:** Advances past malformed tokens until a known recovery point

### Error Reporting with Codes

Errors are categorized by **error codes** for IDE integration:

```rust
fn report_error(&self, message: &str, span: Span) {
    let error_code = if message.contains("break") && message.contains("outside") {
        error_codes::BREAK_OUTSIDE_LOOP  // "E2020"
    } else if message.contains("continue") && message.contains("outside") {
        error_codes::CONTINUE_OUTSIDE_LOOP  // "E2021"
    } else if message.contains("end") {
        error_codes::MISSING_END  // "E2010"
    } else if message.contains("identifier") && message.contains("Expected") {
        error_codes::EXPECTED_IDENTIFIER  // "E2003"
    } else {
        error_codes::UNEXPECTED_TOKEN  // "E2001"
    };

    self.diagnostic_handler.report_error(span, error_code, message);
}
```

### Graceful Degradation

The parser continues after errors to report **multiple issues in one pass:**

```rust
pub fn parse(&mut self) -> Result<Program<'arena>, ParserError> {
    let mut statements = Vec::new();

    while !self.is_at_end() {
        match self.parse_statement() {
            Ok(stmt) => statements.push(stmt),
            Err(e) => {
                self.report_error(&e.message, e.span);
                self.synchronize();  // Skip to next statement
            }
        }
    }

    // Return partial AST even if errors occurred
    Ok(Program::new(self.alloc_vec(statements), /* span */))
}
```

---

## 5. Span Tracking

**Location:** `/crates/luanext-parser/src/span.rs`

### Span Structure

Every AST node tracks its source location:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,   // Byte offset (inclusive)
    pub end: u32,     // Byte offset (exclusive)
    pub line: u32,    // Line number (1-indexed)
    pub column: u32,  // Column number (1-indexed)
}
```

**Memory Efficiency:**
- `u32` fields = 16 bytes per span
- Supports files up to 4GB and 4 billion lines
- Copyable (no heap allocation)

### Span Combining

When creating compound nodes, combine child spans:

```rust
let span = left_expr.span.combine(&right_expr.span);

pub fn combine(&self, other: &Span) -> Span {
    Span {
        start: self.start.min(other.start),
        end: self.end.max(other.end),
        line: self.line.min(other.line),
        column: self.column.min(other.column),
    }
}
```

### Spanned Wrapper

The `Spanned<T>` wrapper adds location to any node:

```rust
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

// Common alias for identifiers
pub type Ident = Spanned<StringId>;
```

### Usage in Error Messages

```rust
diagnostic_handler.report_error(
    expr.span,
    error_codes::TYPE_MISMATCH,
    &format!("Expected number, got {}", actual_type),
);
```

IDE can then:
1. Highlight the exact range `span.start..span.end`
2. Jump to `line:column`
3. Show inline diagnostics

---

## 6. Adding New Syntax

Follow this workflow to add new language features:

### Step 1: Add Lexeme

**File:** `/crates/luanext-parser/src/lexer/lexeme.rs`

```rust
// 1. Add token kind
pub enum TokenKind {
    // ... existing tokens ...
    YourNewKeyword,  // Add here
}

// 2. Add to from_keyword() if it's a keyword
pub fn from_keyword(s: &str) -> Option<TokenKind> {
    match s.len() {
        7 => match s {  // Choose correct length bucket
            "yournew" => Some(TokenKind::YourNewKeyword),
            _ => None,
        },
        // ...
    }
}
```

### Step 2: Add AST Node

**File:** `/crates/luanext-parser/src/ast/<category>.rs`

For a statement:
```rust
// In ast/statement.rs
#[derive(Debug, Clone, Serialize)]
pub struct YourNewStatement<'arena> {
    #[serde(borrow)]
    pub field1: &'arena Expression<'arena>,
    pub field2: Ident,
    pub span: Span,
}

// Add variant to Statement enum
pub enum Statement<'arena> {
    // ... existing variants ...
    YourNew(YourNewStatement<'arena>),
}
```

For an expression:
```rust
// In ast/expression.rs
pub enum ExpressionKind<'arena> {
    // ... existing variants ...
    YourNew(&'arena Expression<'arena>, YourNewData),
}
```

### Step 3: Implement Parser

**File:** `/crates/luanext-parser/src/parser/statement.rs` (or `expression.rs`)

```rust
impl<'a, 'arena> StatementParser<'arena> for Parser<'a, 'arena> {
    fn parse_statement(&mut self) -> Result<Statement<'arena>, ParserError> {
        match &self.current().kind {
            // ... existing cases ...
            TokenKind::YourNewKeyword => self.parse_your_new_statement(),
            // ...
        }
    }
}

impl<'a, 'arena> Parser<'a, 'arena> {
    fn parse_your_new_statement(&mut self) -> Result<Statement<'arena>, ParserError> {
        let start_span = self.current_span();

        // Consume keyword
        self.consume(TokenKind::YourNewKeyword, "Expected 'yournew'")?;

        // Parse components
        let field1 = self.parse_expression()?;
        let field2 = self.parse_identifier()?;

        // Combine spans
        let span = start_span.combine(&field2.span);

        // Allocate in arena
        Ok(Statement::YourNew(YourNewStatement {
            field1: self.alloc(field1),
            field2,
            span,
        }))
    }
}
```

### Step 4: Add Tests

**File:** `/crates/luanext-parser/src/parser/mod.rs` (or dedicated test file)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_new_syntax() {
        let source = "yournew some_value identifier";
        let (program, handler) = parse_program(source);

        assert!(!handler.has_errors());
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0] {
            Statement::YourNew(stmt) => {
                // Verify structure
                assert!(matches!(stmt.field1.kind, ExpressionKind::Identifier(_)));
            }
            _ => panic!("Expected YourNew statement"),
        }
    }

    #[test]
    fn test_your_new_syntax_error_recovery() {
        let source = "yournew some_value";  // Missing required field
        let (program, handler) = parse_program(source);

        assert!(handler.has_errors());
        // Parser should recover and continue
    }
}
```

### Step 5: Update Type Checker (If Needed)

**File:** `/crates/luanext-typechecker/src/checker.rs`

Add type checking logic for the new construct:

```rust
impl TypeChecker {
    fn check_statement(&mut self, stmt: &Statement) -> TypeResult<()> {
        match stmt {
            Statement::YourNew(your_new) => {
                self.check_your_new_statement(your_new)?;
            }
            // ... other cases ...
        }
        Ok(())
    }
}
```

### Step 6: Update Code Generator (If Needed)

**File:** `/crates/luanext-core/src/codegen/<module>.rs`

Add Lua output generation for the new syntax.

---

## 7. AST Node Design Patterns

### Pattern 1: Arena-Allocated References

All child nodes use **arena references** (`&'arena T`):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct IfStatement<'arena> {
    #[serde(borrow)]  // Tell serde to borrow during serialization
    pub condition: Expression<'arena>,
    #[serde(borrow)]
    pub then_block: Block<'arena>,
    #[serde(borrow)]
    pub else_ifs: &'arena [ElseIf<'arena>],  // Slices, not Vec
    #[serde(borrow)]
    pub else_block: Option<Block<'arena>>,
    pub span: Span,
}
```

**Why Slices?**
- `&'arena [T]` is owned by the arena, not the AST struct
- No `Drop` implementation needed
- Covariant lifetime (safe to shorten `'arena`)

### Pattern 2: Spanned Identifiers

Identifiers are always `Spanned<StringId>`:

```rust
pub type Ident = Spanned<StringId>;

// Usage
let var_name: Ident = self.parse_identifier()?;
let name_str = self.interner.resolve(var_name.node);
```

### Pattern 3: Enum Discriminants for Variants

Use `#[serde(borrow)]` and `#[serde(default)]` for optional fields:

```rust
#[derive(Debug, Clone, Serialize)]
pub enum Statement<'arena> {
    #[serde(borrow)]
    Variable(VariableStatement<'arena>),
    #[serde(borrow)]
    Function(FunctionStatement<'arena>),
    Break(Span),  // Simple variants carry just a span
    Continue(Span),
}
```

### Pattern 4: Default Trait for Expressions

Expressions have optional metadata fields:

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct Expression<'arena> {
    pub kind: ExpressionKind<'arena>,
    pub span: Span,
    #[serde(borrow)]
    pub annotated_type: Option<Type<'arena>>,  // Filled by type checker
    pub receiver_class: Option<ReceiverClassInfo>,  // For method calls
}
```

`Default` allows clean construction:

```rust
Expression {
    kind: ExpressionKind::Binary(/* ... */),
    span,
    ..Default::default()  // Sets annotated_type and receiver_class to None
}
```

### Pattern 5: Recursive Types

Self-referential types use `Box` or arena references:

```rust
pub enum Type<'arena> {
    // Base types
    Primitive(PrimitiveType),
    Reference(StringId),

    // Recursive types (arena-allocated)
    #[serde(borrow)]
    Array(&'arena Type<'arena>),
    #[serde(borrow)]
    Function(FunctionType<'arena>),
    #[serde(borrow)]
    Union(&'arena [Type<'arena>]),
}
```

---

## 8. Testing Parser Changes

### Unit Test Pattern

**Location:** Inline `#[cfg(test)]` modules in parser files

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::CollectingDiagnosticHandler;
    use std::sync::Arc;

    fn parse_expr(source: &str) -> Expression<'static> {
        let arena = Box::leak(Box::new(Bump::new()));
        let handler = Arc::new(CollectingDiagnosticHandler::new());
        let (interner, common) = StringInterner::new_with_common_identifiers();

        let mut lexer = Lexer::new(source, handler.clone(), &interner);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(tokens, handler, &interner, &common, arena);
        parser.parse_expression().unwrap()
    }

    #[test]
    fn test_binary_precedence() {
        let expr = parse_expr("1 + 2 * 3");

        // Should parse as 1 + (2 * 3), not (1 + 2) * 3
        match expr.kind {
            ExpressionKind::Binary(BinaryOp::Add, left, right) => {
                assert!(matches!(left.kind, ExpressionKind::Literal(Literal::Integer(1))));
                assert!(matches!(right.kind, ExpressionKind::Binary(BinaryOp::Multiply, _, _)));
            }
            _ => panic!("Incorrect precedence"),
        }
    }
}
```

### Integration Tests

**Location:** `/crates/luanext-cli/tests/`

Test full compilation pipeline:

```rust
#[test]
fn test_new_syntax_integration() {
    let source = r#"
        yournew expression {
            field: value
        }
    "#;

    let result = compile_source(source);
    assert!(result.is_ok());

    let lua_output = result.unwrap();
    assert!(lua_output.contains("expected_lua_code"));
}
```

### Fuzz Testing

**Location:** `/crates/luanext-parser/fuzz/fuzz_targets/`

Fuzzer finds edge cases automatically:

```rust
// fuzz_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use luanext_parser::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let _ = parse_with_container(source, &mut container, &arena);
        // Should not panic, even on garbage input
    }
});
```

Run with: `cargo fuzz run fuzz_parser`

### Property-Based Testing

Use `proptest` for generating valid syntax:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_identifier(s in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
        let expr = parse_expr(&s);
        match expr.kind {
            ExpressionKind::Identifier(id) => {
                assert_eq!(interner.resolve(id), s);
            }
            _ => prop_assert!(false, "Should parse as identifier"),
        }
    }
}
```

### Benchmark Tests

**Location:** `/crates/luanext-parser/benches/`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_expression(c: &mut Criterion) {
    let source = "1 + 2 * 3 - 4 / 5";

    c.bench_function("parse_expr", |b| {
        b.iter(|| {
            let expr = parse_expr(black_box(source));
            black_box(expr);
        });
    });
}

criterion_group!(benches, bench_parse_expression);
criterion_main!(benches);
```

Run with: `cargo bench --package luanext-parser`

### Coverage Analysis

Generate coverage reports with `tarpaulin`:

```bash
cargo tarpaulin --workspace --exclude-files "fuzz/*" --out Html
```

Target: **70%+ coverage** for parser code.

---

## Common Pitfalls

### 1. Forgetting to Advance

```rust
// ❌ Infinite loop - forgot to advance
while self.check(&TokenKind::Comma) {
    let item = self.parse_item()?;
    items.push(item);
}

// ✅ Correct
while self.match_token(&[TokenKind::Comma]) {  // Advances if matched
    let item = self.parse_item()?;
    items.push(item);
}
```

### 2. Incorrect Span Combination

```rust
// ❌ Wrong - end_span is not the last token
let span = start_span.combine(&some_middle_token.span);

// ✅ Correct - combine with actual last token
self.consume(TokenKind::End, "Expected 'end'")?;
let end_span = self.current_span();
let span = start_span.combine(&end_span);
```

### 3. Not Using Arena Allocation

```rust
// ❌ Heap allocation - lifetime issues
let children = vec![child1, child2];
Node { children }  // Vec<T> owns data, can't use 'arena lifetime

// ✅ Arena allocation
let children = self.alloc_vec(vec![child1, child2]);
Node { children }  // &'arena [T] is arena-owned
```

### 4. Breaking Precedence Order

```rust
// ❌ Wrong - or should call and, not vice versa
fn parse_logical_or(&mut self) -> Result<Expression<'arena>, ParserError> {
    let mut expr = self.parse_comparison()?;  // Skipped several levels!
    // ...
}

// ✅ Correct - maintain precedence chain
fn parse_logical_or(&mut self) -> Result<Expression<'arena>, ParserError> {
    let mut expr = self.parse_null_coalesce()?;  // Next lower precedence
    // ...
}
```

### 5. Poor Error Messages

```rust
// ❌ Vague
self.consume(TokenKind::RightParen, "Expected ')'")?;

// ✅ Contextual
self.consume(TokenKind::RightParen, "Expected ')' after function arguments")?;
```

---

## Summary

The LuaNext parser is a **recursive descent parser with Pratt-style precedence** that:

1. **Lexer:** Converts source to tokens with string interning and performance optimizations
2. **Parser:** Uses modular trait-based design with arena allocation for AST nodes
3. **Precedence:** Implements 15 precedence levels via recursive functions
4. **Error Recovery:** Synchronizes to statement boundaries and reports multiple errors
5. **Span Tracking:** Maintains precise source locations for IDE integration
6. **Extensible:** Clear patterns for adding new syntax with comprehensive testing

**Key Files:**
- Lexer: `/crates/luanext-parser/src/lexer/mod.rs`
- Parser Core: `/crates/luanext-parser/src/parser/mod.rs`
- Expressions: `/crates/luanext-parser/src/parser/expression.rs`
- Statements: `/crates/luanext-parser/src/parser/statement.rs`
- AST Definitions: `/crates/luanext-parser/src/ast/`
- Span Tracking: `/crates/luanext-parser/src/span.rs`

Agent is calibrated...
