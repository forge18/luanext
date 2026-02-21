# Abstract Syntax Tree

Defines every node in the LuaNext AST. The parser produces a `Program` containing a slice of `Statement` nodes, each recursively composed of expressions, patterns, types, and supporting structures. All AST nodes are arena-allocated using `bumpalo::Bump` and carry lifetime `'arena`.

## Overview

**Source**: `crates/luanext-parser/src/ast/` (4 modules: `mod.rs`, `statement.rs`, `expression.rs`, `pattern.rs`, `types.rs`)

The AST is the canonical intermediate representation between parsing and all downstream phases (type checking, codegen, optimization, LSP). Every node carries a `Span` for source location tracking. Identifier strings are stored as `StringId` values from the string interner, not raw strings.

### Design Principles

- **Arena allocation**: All child slices and recursive references use `&'arena` borrows from a `bumpalo::Bump` arena. No heap allocation via `Box` or `Vec` in the tree structure (except `EnumValue::String`, `NamespaceDeclaration::path`, `ImportDeclaration::source`, and `Namespace` TypeKind).
- **Serializable**: All nodes derive `Serialize` for cache and debug output. `Deserialize` is only on select leaf types (`PrimitiveType`, `IndexKeyType`, `StringId`).
- **Span tracking**: Every node either contains a `span: Span` field or is wrapped in a `Spanned<T>` container.

## Program (Root Node)

```rust
struct Program<'arena> {
    statements: &'arena [Statement<'arena>],
    span: Span,
    statement_ranges: Option<Vec<(usize, Span)>>,  // skip serialization
}
```

| Field | Purpose |
| ----- | ------- |
| `statements` | Top-level statement list |
| `span` | Span covering the entire source file |
| `statement_ranges` | Index-to-span mapping for incremental parsing; computed on construction, skipped during serialization |

`Program::new()` automatically populates `statement_ranges` by extracting spans from each statement.

## Spanned and Ident

```rust
struct Spanned<T> {
    node: T,
    span: Span,
}

type Ident = Spanned<StringId>;
```

`Ident` is the standard identifier node used throughout the AST. It pairs a `StringId` (interned string) with a `Span`.

---

## Statements

`Statement<'arena>` is the top-level enum for all statement-level constructs. The variants are grouped by category below.

### Core Declarations

#### Variable

```lua
Statement::Variable(VariableDeclaration)
```

```rust
struct VariableDeclaration<'arena> {
    kind: VariableKind,             // Const | Global | Local
    pattern: Pattern<'arena>,       // binding target (simple name or destructuring)
    type_annotation: Option<Type>,  // optional type annotation
    initializer: Expression,        // right-hand side value
    span: Span,
}
```

```rust
enum VariableKind { Const, Global, Local }
```

**Syntax examples**:

```lua
const x = 42              -- Const, Identifier pattern
local [a, b] = getValues() -- Local, Array pattern
global config: Config = {} -- Global, Identifier pattern with type annotation
x: number = 42            -- Global (implicit), with type annotation
```

`[NOTE: BEHAVIOR]` `VariableKind::Global` produces Lua code without the `local` prefix. `Const` and `Local` both emit `local`. The difference between `Const` and `Local` is enforced by the type checker (const prevents reassignment), not codegen.

`[NOTE: BEHAVIOR]` Implicit global syntax (`x: number = 42`) is disambiguated from type assertions (`x :: T`) by parser lookahead: `Identifier : <not Colon>` triggers a global declaration parse.

#### Function

```lua
Statement::Function(FunctionDeclaration)
```

```rust
struct FunctionDeclaration<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    parameters: &'arena [Parameter],
    return_type: Option<Type>,
    throws: Option<&'arena [Type]>,  // declared throwable types
    body: Block,
    span: Span,
}
```

**Syntax**:

```lua
function greet<T>(name: string): string throws Error {
    return `Hello ${name}`
}
```

#### Class

```lua
Statement::Class(ClassDeclaration)
```

```rust
struct ClassDeclaration<'arena> {
    decorators: &'arena [Decorator],
    is_abstract: bool,
    is_final: bool,
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    primary_constructor: Option<&'arena [ConstructorParameter]>,
    extends: Option<Type>,
    parent_constructor_args: Option<&'arena [Expression]>,
    implements: &'arena [Type],
    members: &'arena [ClassMember],
    is_forward_declaration: bool,
    span: Span,
}
```

**Syntax**:

```lua
@serializable
abstract class Animal<T> extends LivingThing(baseArgs) implements Printable {
    // members
}

-- Primary constructor (compact syntax)
class Point(public x: number, public y: number) {
    // auto-generates fields + constructor
}
```

`[NOTE: BEHAVIOR]` `is_forward_declaration` marks classes with no members, used for mutual/circular type references. Forward declarations have `serde(default)` for backward-compatible deserialization.

##### Class Members

```rust
enum ClassMember<'arena> {
    Property(PropertyDeclaration),
    Constructor(ConstructorDeclaration),
    Method(MethodDeclaration),
    Getter(GetterDeclaration),
    Setter(SetterDeclaration),
    Operator(OperatorDeclaration),
}
```

**PropertyDeclaration**:

```rust
struct PropertyDeclaration<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    is_static: bool,
    is_readonly: bool,
    name: Ident,
    type_annotation: Type,    // required (not optional)
    initializer: Option<Expression>,
    span: Span,
}
```

**ConstructorDeclaration**:

```rust
struct ConstructorDeclaration<'arena> {
    decorators: &'arena [Decorator],
    parameters: &'arena [Parameter],
    body: Block,
    span: Span,
}
```

**ConstructorParameter** (for primary constructors):

```rust
struct ConstructorParameter<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    is_readonly: bool,
    name: Ident,
    type_annotation: Type,     // required
    default: Option<Expression>,
    span: Span,
}
```

**MethodDeclaration**:

```rust
struct MethodDeclaration<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    is_static: bool,
    is_abstract: bool,
    is_final: bool,
    is_override: bool,
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    parameters: &'arena [Parameter],
    return_type: Option<Type>,
    body: Option<Block>,   // None for abstract methods
    span: Span,
}
```

`[NOTE: BEHAVIOR]` Abstract methods have `body: None`. Abstract/final validation is enforced at type-check time, not runtime.

**GetterDeclaration**:

```rust
struct GetterDeclaration<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    is_static: bool,
    name: Ident,
    return_type: Type,    // required
    body: Block,
    span: Span,
}
```

**SetterDeclaration**:

```rust
struct SetterDeclaration<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    is_static: bool,
    name: Ident,
    parameter: Parameter,  // single parameter (not a slice)
    body: Block,
    span: Span,
}
```

`[NOTE: BEHAVIOR]` Codegen maps `get X()` to `get_X()` and `set X(v)` to `set_X(v)`. Access control registers them under the original name `X`. The type checker handles this with prefix-stripping fallback in `infer_method()` / `infer_member()`.

**OperatorDeclaration**:

```rust
struct OperatorDeclaration<'arena> {
    decorators: &'arena [Decorator],
    access: Option<AccessModifier>,
    operator: OperatorKind,
    parameters: &'arena [Parameter],
    return_type: Option<Type>,
    body: Block,
    span: Span,
}
```

**OperatorKind** (25 overloadable operators):

```rust
enum OperatorKind {
    Add, Subtract, Multiply, Divide, Modulo, Power, Concatenate, FloorDivide,
    Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual,
    BitwiseAnd, BitwiseOr, BitwiseXor, ShiftLeft, ShiftRight,
    Index, NewIndex, Call,
    UnaryMinus, Length,
}
```

##### Access Modifiers

```rust
enum AccessModifier { Public, Private, Protected }
```

Default access (when `access` is `None`) is public.

#### Interface

```lua
Statement::Interface(InterfaceDeclaration)
```

```rust
struct InterfaceDeclaration<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    extends: &'arena [Type],         // multiple interface extension
    members: &'arena [InterfaceMember],
    is_forward_declaration: bool,
    span: Span,
}
```

```rust
enum InterfaceMember<'arena> {
    Property(PropertySignature),
    Method(MethodSignature),
    Index(IndexSignature),
}
```

**PropertySignature**:

```rust
struct PropertySignature<'arena> {
    is_readonly: bool,
    name: Ident,
    is_optional: bool,
    type_annotation: Type,
    span: Span,
}
```

**MethodSignature**:

```rust
struct MethodSignature<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    parameters: &'arena [Parameter],
    return_type: Type,               // required
    body: Option<Block>,             // default implementation
    span: Span,
}
```

`[NOTE: BEHAVIOR]` Interface methods can have a `body` for default implementations.

**IndexSignature**:

```rust
struct IndexSignature<'arena> {
    key_name: Ident,
    key_type: IndexKeyType,   // String | Number
    value_type: Type,
    span: Span,
}
```

```rust
enum IndexKeyType { String, Number }
```

**Syntax**:

```lua
interface Printable<T> extends Serializable {
    readonly name: string
    greet(target: string): string
    [key: string]: unknown
}
```

#### TypeAlias

```lua
Statement::TypeAlias(TypeAliasDeclaration)
```

```rust
struct TypeAliasDeclaration<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    type_annotation: Type,
    span: Span,
}
```

**Syntax**: `type StringOrNumber = string | number`

`[NOTE: BEHAVIOR]` Type aliases are erased during codegen. They produce no Lua output.

#### Enum

```lua
Statement::Enum(EnumDeclaration)
```

```rust
struct EnumDeclaration<'arena> {
    name: Ident,
    members: &'arena [EnumMember],
    fields: &'arena [EnumField],
    constructor: Option<EnumConstructor>,
    methods: &'arena [EnumMethod],
    implements: &'arena [Type],
    span: Span,
}
```

**EnumMember**:

```rust
struct EnumMember<'arena> {
    name: Ident,
    arguments: &'arena [Expression],  // constructor arguments (rich enums)
    value: Option<EnumValue>,         // explicit value (simple enums)
    span: Span,
}

enum EnumValue {
    Number(f64),
    String(String),
}
```

**EnumField**: `{ name: Ident, type_annotation: Type, span: Span }`

**EnumConstructor**: `{ parameters: &'arena [Parameter], body: Block, span: Span }`

**EnumMethod**: `{ name: Ident, parameters: &'arena [Parameter], return_type: Option<Type>, body: Block, span: Span }`

Enums support two forms:

- **Simple enums**: members have optional `EnumValue` (auto-numbered if omitted)
- **Rich enums**: members have constructor `arguments`, and the enum declares `fields`, a `constructor`, and `methods`

**Syntax**:

```lua
-- Simple
enum Color { Red, Green = 1, Blue }

-- Rich
enum Planet implements Describable {
    field mass: number
    field radius: number

    constructor(mass: number, radius: number) {
        self.mass = mass
        self.radius = radius
    }

    Earth(5.97e24, 6.37e6),
    Mars(6.42e23, 3.39e6)

    function describe(): string {
        return `mass=${self.mass}`
    }
}
```

### Control Flow

#### If

```lua
Statement::If(IfStatement)
```

```rust
struct IfStatement<'arena> {
    condition: Expression,
    then_block: Block,
    else_ifs: &'arena [ElseIf],
    else_block: Option<Block>,
    span: Span,
}

struct ElseIf<'arena> {
    condition: Expression,
    block: Block,
    span: Span,
}
```

**Syntax**: Supports both brace and Lua-style block syntax.

```lua
if x > 0 {
    print("positive")
} elseif x == 0 {
    print("zero")
} else {
    print("negative")
}
```

`[NOTE: BEHAVIOR]` LuaNext uses EITHER braces `{ }` OR `do/then ... end` for blocks -- never mixed within one statement. For example, `if x { ... end` is invalid.

#### While

```lua
Statement::While(WhileStatement)
```

```rust
struct WhileStatement<'arena> {
    condition: Expression,
    body: Block,
    span: Span,
}
```

#### For

```lua
Statement::For(&'arena ForStatement)
```

`[NOTE: ALLOCATION]` `ForStatement` is allocated behind an `&'arena` reference (not inlined), unlike most other statement payloads.

```rust
enum ForStatement<'arena> {
    Numeric(&'arena ForNumeric),
    Generic(ForGeneric),
}
```

**ForNumeric**:

```rust
struct ForNumeric<'arena> {
    variable: Ident,
    start: Expression,
    end: Expression,
    step: Option<Expression>,
    body: Block,
    span: Span,
}
```

**Syntax**: `for i = 1, 10, 2 do ... end`

**ForGeneric**:

```rust
struct ForGeneric<'arena> {
    variables: &'arena [Ident],
    pattern: Option<Pattern>,    // destructuring pattern
    iterators: &'arena [Expression],
    body: Block,
    span: Span,
}
```

**Syntax**:

```lua
for k, v in pairs(t) do ... end
for [x, y] in points do ... end   -- destructuring
```

`[NOTE: BEHAVIOR]` For-in destructuring requires typed arrays (e.g., `number[][]` or `{x: number}[]`) for the type checker to resolve element types.

#### Repeat

```lua
Statement::Repeat(RepeatStatement)
```

```rust
struct RepeatStatement<'arena> {
    body: Block,
    until: Expression,
    span: Span,
}
```

**Syntax**: `repeat ... until condition`

#### Return

```lua
Statement::Return(ReturnStatement)
```

```rust
struct ReturnStatement<'arena> {
    values: &'arena [Expression],  // multiple return values
    span: Span,
}
```

**Syntax**: `return a, b, c` (Lua-style multiple returns)

#### Break and Continue

```lua
Statement::Break(Span)
Statement::Continue(Span)
```

Simple span-only statements with no payload.

`[NOTE: BEHAVIOR]` `continue` emits Lua 5.5 native `continue` syntax. It is not available on earlier Lua targets.

#### Label and Goto

```lua
Statement::Label(LabelStatement)   -- { name: Ident, span: Span }
Statement::Goto(GotoStatement)     -- { target: Ident, span: Span }
```

**Syntax**: `::label_name::` and `goto label_name`

### Module System

#### Import

```lua
Statement::Import(ImportDeclaration)
```

```rust
struct ImportDeclaration<'arena> {
    clause: ImportClause,
    source: String,       // module path string (heap-allocated)
    span: Span,
}
```

```rust
enum ImportClause<'arena> {
    Default(Ident),
    Named(&'arena [ImportSpecifier]),
    Namespace(Ident),
    TypeOnly(&'arena [ImportSpecifier]),
    Mixed { default: Ident, named: &'arena [ImportSpecifier] },
}

struct ImportSpecifier {
    imported: Ident,        // name in source module
    local: Option<Ident>,   // renamed binding (alias)
    span: Span,
}
```

**Syntax**:

```lua
import Foo from "./foo"                       -- Default
import { bar, baz as qux } from "./utils"    -- Named
import * as Utils from "./utils"              -- Namespace
import type { MyType } from "./types"         -- TypeOnly
import Foo, { bar } from "./foo"              -- Mixed
```

`[NOTE: BEHAVIOR]` `ImportClause::TypeOnly` imports are erased during codegen (produce no Lua output). The type checker validates that type-only imports are not used as runtime values via `validate_import_export_compatibility()`.

#### Export

```lua
Statement::Export(ExportDeclaration)
```

```rust
struct ExportDeclaration<'arena> {
    kind: ExportKind,
    span: Span,
}

enum ExportKind<'arena> {
    Declaration(&'arena Statement),       // export function/class/etc.
    Named {
        specifiers: &'arena [ExportSpecifier],
        source: Option<String>,           // re-export source
        is_type_only: bool,
    },
    Default(&'arena Expression),          // export default expr
    All {
        source: String,                   // export * from "source"
        is_type_only: bool,
    },
}

struct ExportSpecifier {
    local: Ident,
    exported: Option<Ident>,   // rename on export
    span: Span,
}
```

**Syntax**:

```lua
export function greet() { ... }                -- Declaration
export { foo, bar as baz }                     -- Named
export default MyClass                         -- Default
export * from "./utils"                        -- All (re-export)
export { thing } from "./other"                -- Named re-export
export type { MyType } from "./types"          -- Type-only re-export
```

`[NOTE: BEHAVIOR]` Re-exports (`Named` with `source` or `All`) are resolved by the type checker with cycle detection (max depth 10) and proper type-only validation. See `resolve_re_export()` in the type checker.

#### Namespace

```lua
Statement::Namespace(NamespaceDeclaration)
```

```rust
struct NamespaceDeclaration {
    path: Vec<Ident>,    // dot-separated path segments
    span: Span,
}
```

**Syntax**: `namespace My.Utils.Math`

`[NOTE: ALLOCATION]` `path` uses `Vec<Ident>` (heap-allocated), one of the few non-arena allocations in the AST.

### Exception Handling

#### Throw

```lua
Statement::Throw(ThrowStatement)
```

```rust
struct ThrowStatement<'arena> {
    expression: Expression,
    span: Span,
}
```

**Syntax**: `throw Error("something failed")`

#### Try

```lua
Statement::Try(TryStatement)
```

```rust
struct TryStatement<'arena> {
    try_block: Block,
    catch_clauses: &'arena [CatchClause],
    finally_block: Option<Block>,
    span: Span,
}

struct CatchClause<'arena> {
    pattern: CatchPattern,
    body: Block,
    span: Span,
}

enum CatchPattern<'arena> {
    Untyped { variable: Ident, span: Span },
    Typed { variable: Ident, type_annotation: Type, span: Span },
    MultiTyped { variable: Ident, type_annotations: &'arena [Type], span: Span },
}
```

**Syntax**:

```lua
try {
    riskyOperation()
} catch (e) {
    -- untyped catch
} catch (e: TypeError) {
    -- typed catch
} catch (e: IOError | NetworkError) {
    -- multi-typed catch
} finally {
    cleanup()
}
```

`[NOTE: BEHAVIOR]` Codegen transforms `try`/`catch` into Lua `pcall` wrappers. The catch body runs inside `if not __ok then` -- there is no `else` keyword between the pcall result check and catch body.

#### Rethrow

```lua
Statement::Rethrow(Span)
```

Re-raises the current exception from within a catch block. Span-only, no payload.

**Syntax**: `rethrow`

### Other Statements

#### Expression Statement

```lua
Statement::Expression(Expression)
```

Any expression used as a statement (function calls, assignments, etc.).

#### Block

```lua
Statement::Block(Block)
```

```rust
struct Block<'arena> {
    statements: &'arena [Statement],
    span: Span,
}
```

A block of statements. Used both standalone and as the body of control flow, functions, classes, etc.

#### Multi-Assignment

```lua
Statement::MultiAssignment(MultiAssignmentStatement)
```

```rust
struct MultiAssignmentStatement<'arena> {
    targets: &'arena [Expression],   // left-hand side (existing variables)
    values: &'arena [Expression],    // right-hand side
    span: Span,
}
```

**Syntax**: `a, b, c = expr1, expr2, expr3`

Used for assigning to existing variables (not declarations) in parallel, including capturing multiple return values. This is distinct from `VariableDeclaration` which introduces new bindings.

### Declaration File Statements

These appear in `.d.luax` declaration files to describe external Lua library types without implementation bodies.

#### DeclareFunction

```lua
Statement::DeclareFunction(DeclareFunctionStatement)
```

```rust
struct DeclareFunctionStatement<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter]>,
    parameters: &'arena [Parameter],
    return_type: Type,       // required (not optional)
    throws: Option<&'arena [Type]>,
    is_export: bool,         // for `export function` inside declare namespaces
    span: Span,
}
```

`[NOTE: BEHAVIOR]` Unlike `FunctionDeclaration`, the return type is required (not `Option`) and there is no body.

#### DeclareNamespace

```lua
Statement::DeclareNamespace(DeclareNamespaceStatement)
```

```rust
struct DeclareNamespaceStatement<'arena> {
    name: Ident,
    members: &'arena [Statement],   // can contain nested declarations
    span: Span,
}
```

Members can include `DeclareFunction`, `DeclareConst`, nested `DeclareNamespace`, `DeclareType`, and `DeclareInterface`.

#### DeclareType and DeclareInterface

```lua
Statement::DeclareType(TypeAliasDeclaration)
Statement::DeclareInterface(InterfaceDeclaration)
```

Reuse the same AST nodes as `TypeAlias` and `Interface` respectively. The `Statement` variant distinguishes their origin in declaration files.

#### DeclareConst

```lua
Statement::DeclareConst(DeclareConstStatement)
```

```rust
struct DeclareConstStatement<'arena> {
    name: Ident,
    type_annotation: Type,   // required
    is_export: bool,
    span: Span,
}
```

---

## Expressions

`Expression<'arena>` is a struct wrapping `ExpressionKind` with metadata:

```rust
struct Expression<'arena> {
    kind: ExpressionKind,
    span: Span,
    annotated_type: Option<Type>,              // set by type checker
    receiver_class: Option<ReceiverClassInfo>,  // set by type checker for method calls
}

struct ReceiverClassInfo {
    class_name: StringId,
    is_static: bool,
}
```

`[NOTE: BEHAVIOR]` `annotated_type` and `receiver_class` are `None` after parsing. The type checker populates these during type inference. `ExpressionKind::SelfKeyword` is the `#[default]` variant for `Default` derivation.

### Identifiers and Literals

```lua
ExpressionKind::Identifier(StringId)
ExpressionKind::Literal(Literal)
ExpressionKind::SelfKeyword
ExpressionKind::SuperKeyword
```

```rust
enum Literal {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
}
```

`[NOTE: BEHAVIOR]` `Number(f64)` and `Integer(i64)` are distinct. The parser produces `Integer` for whole numbers without decimal points, `Number` for floating-point literals.

### Binary Operations

```lua
ExpressionKind::Binary(BinaryOp, &'arena Expression, &'arena Expression)
```

```rust
enum BinaryOp {
    // Arithmetic
    Add, Subtract, Multiply, Divide, Modulo, IntegerDivide, Power,
    // Comparison
    Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual,
    // Logical
    And, Or,
    // Null coalescing
    NullCoalesce,
    // String
    Concatenate,
    // Bitwise
    BitwiseAnd, BitwiseOr, BitwiseXor, ShiftLeft, ShiftRight,
    // Type checking
    Instanceof,
}
```

22 binary operators total.

`[NOTE: BEHAVIOR]` `And`/`Or` are typed as `Boolean` by the type checker's inference. `NullCoalesce` (`??`) returns the left operand if non-nil, otherwise the right. `Instanceof` checks runtime type membership.

### Unary Operations

```lus
ExpressionKind::Unary(UnaryOp, &'arena Expression)
```

```rust
enum UnaryOp { Not, Negate, Length, BitwiseNot }
```

| Operator | Syntax | Description |
| -------- | ------ | ----------- |
| `Not` | `not x` | Logical negation |
| `Negate` | `-x` | Arithmetic negation |
| `Length` | `#x` | Length operator |
| `BitwiseNot` | `~x` | Bitwise complement |

### Assignment

```lua
ExpressionKind::Assignment(&'arena Expression, AssignmentOp, &'arena Expression)
```

```rust
enum AssignmentOp {
    Assign,            // =
    AddAssign,         // +=
    SubtractAssign,    // -=
    MultiplyAssign,    // *=
    DivideAssign,      // /=
    ModuloAssign,      // %=
    PowerAssign,       // ^=
    ConcatenateAssign, // ..=
    BitwiseAndAssign,  // &=
    BitwiseOrAssign,   // |=
    FloorDivideAssign, // //=
    LeftShiftAssign,   // <<=
    RightShiftAssign,  // >>=
}
```

13 assignment operators. The left-hand side is an expression (variable, member access, or index).

`[NOTE: BEHAVIOR]` Compound assignments (e.g., `+=`) are desugared by codegen into `x = x + value` in the output Lua.

### Member Access and Indexing

```lua
ExpressionKind::Member(&'arena Expression, Ident)       -- obj.field
ExpressionKind::Index(&'arena Expression, &'arena Expression)  -- obj[key]
```

### Function Calls

```lua
ExpressionKind::Call(&'arena Expression, &'arena [Argument], Option<&'arena [Type]>)
ExpressionKind::MethodCall(&'arena Expression, Ident, &'arena [Argument], Option<&'arena [Type]>)
```

```rust
struct Argument<'arena> {
    value: Expression,
    is_spread: bool,
    span: Span,
}
```

**Syntax**:

```lua
foo(1, 2, ...args)            -- Call with spread
obj::method<string>(arg)      -- MethodCall with type arguments
```

The optional `Type` slice carries explicit type arguments for generic calls.

### Constructor (New)

```lua
ExpressionKind::New(&'arena Expression, &'arena [Argument], Option<&'arena [Type]>)
```

**Syntax**: `new MyClass<T>(arg1, arg2)`

### Array and Object Literals

```lua
ExpressionKind::Array(&'arena [ArrayElement])
ExpressionKind::Object(&'arena [ObjectProperty])
```

```rust
enum ArrayElement<'arena> {
    Expression(Expression),
    Spread(Expression),
}

enum ObjectProperty<'arena> {
    Property { key: Ident, value: &'arena Expression, span: Span },
    Computed { key: &'arena Expression, value: &'arena Expression, span: Span },
    Spread { value: &'arena Expression, span: Span },
}
```

**Syntax**:

```lua
[1, 2, ...rest]
{ name = "hello", [computedKey] = value, ...defaults }
```

`[NOTE: BEHAVIOR]` Keywords can be used as property keys in object literals (e.g., `{ get = 42, type = "foo" }`). The parser checks for keyword tokens followed by `=` or `:` in this context.

### Function Expressions

```lua
ExpressionKind::Function(FunctionExpression)
ExpressionKind::Arrow(ArrowFunction)
```

```rust
struct FunctionExpression<'arena> {
    type_parameters: Option<&'arena [TypeParameter]>,
    parameters: &'arena [Parameter],
    return_type: Option<Type>,
    body: Block,
    span: Span,
}

struct ArrowFunction<'arena> {
    parameters: &'arena [Parameter],
    return_type: Option<Type>,
    body: ArrowBody,
    span: Span,
}

enum ArrowBody<'arena> {
    Expression(&'arena Expression),
    Block(Block),
}
```

**Syntax**:

```lua
function(x: number): number { return x * 2 }
(x) => x * 2            -- expression body
(x) => { return x * 2 } -- block body
```

`[NOTE: BEHAVIOR]` Arrow functions do not support type parameters (unlike `FunctionExpression`).

### Conditional (Ternary)

```lua
ExpressionKind::Conditional(&'arena Expression, &'arena Expression, &'arena Expression)
```

**Syntax**: `condition ? trueExpr : falseExpr`

Fields: `(condition, consequent, alternate)`.

### Pipe Operator

```lua
ExpressionKind::Pipe(&'arena Expression, &'arena Expression)
```

**Syntax**: `value |> transform`

Fields: `(input, function)`.

### Match Expression

```lua
ExpressionKind::Match(MatchExpression)
```

```rust
struct MatchExpression<'arena> {
    value: &'arena Expression,
    arms: &'arena [MatchArm],
    span: Span,
}

struct MatchArm<'arena> {
    pattern: Pattern,
    guard: Option<Expression>,
    body: MatchArmBody,
    span: Span,
}

enum MatchArmBody<'arena> {
    Expression(&'arena Expression),
    Block(Block),
}
```

**Syntax**:

```lua
match value {
    1 | 2 => "small",
    n when n > 100 => "large",
    _ => "medium"
}
```

### Template Literals

```lua
ExpressionKind::Template(TemplateLiteral)
```

```rust
struct TemplateLiteral<'arena> {
    parts: &'arena [TemplatePart],
    span: Span,
}

enum TemplatePart<'arena> {
    String(String),
    Expression(&'arena Expression),
}
```

**Syntax**: `` `Hello ${name}, you are ${age} years old` ``

### Type Assertion

```lua
ExpressionKind::TypeAssertion(&'arena Expression, Type)
```

**Syntax**: `expr as Type`

### Optional Chaining

```lua
ExpressionKind::OptionalMember(&'arena Expression, Ident)
ExpressionKind::OptionalIndex(&'arena Expression, &'arena Expression)
ExpressionKind::OptionalCall(&'arena Expression, &'arena [Argument], Option<&'arena [Type]>)
ExpressionKind::OptionalMethodCall(&'arena Expression, Ident, &'arena [Argument], Option<&'arena [Type]>)
```

**Syntax**: `obj?.field`, `obj?[key]`, `fn?(args)`, `obj?.::method(args)`

Each mirrors the non-optional equivalent but short-circuits to `nil` when the receiver is `nil`.

### Try Expression

```lua
ExpressionKind::Try(TryExpression)
```

```rust
struct TryExpression<'arena> {
    expression: &'arena Expression,
    catch_variable: Ident,
    catch_expression: &'arena Expression,
    span: Span,
}
```

**Syntax**: `try expr catch (e) fallbackExpr`

An inline try-catch that evaluates to the expression's result on success, or the catch expression on failure.

### Error Chain

```lua
ExpressionKind::ErrorChain(&'arena Expression, &'arena Expression)
```

Error chain operator for propagating errors through expression chains.

### Parenthesized

```lua
ExpressionKind::Parenthesized(&'arena Expression)
```

Preserves explicit parenthesization in the AST for faithful source representation and correct precedence in codegen.

---

## Patterns

Patterns are used in variable bindings, function parameters, match arms, and for-in destructuring.

```rust
enum Pattern<'arena> {
    Identifier(Ident),
    Literal(Literal, Span),
    Array(ArrayPattern),
    Object(ObjectPattern),
    Wildcard(Span),
    Or(OrPattern),
    Template(TemplatePattern),
}
```

### Identifier Pattern

```lua
Pattern::Identifier(Ident)
```

Simple name binding. The most common pattern.

### Literal Pattern

```lua
Pattern::Literal(Literal, Span)
```

Matches a constant value in match arms. Uses the same `Literal` enum as expressions.

### Array Pattern (Destructuring)

```lua
Pattern::Array(ArrayPattern)
```

```rust
struct ArrayPattern<'arena> {
    elements: &'arena [ArrayPatternElement],
    span: Span,
}

enum ArrayPatternElement<'arena> {
    Pattern(PatternWithDefault),
    Rest(Ident),
    Hole,
}

struct PatternWithDefault<'arena> {
    pattern: Pattern,
    default: Option<Expression>,
}
```

**Syntax**: `const [a, , b = 10, ...rest] = values`

- `Pattern`: an element with optional default value
- `Rest(Ident)`: `...rest` captures remaining elements
- `Hole`: `,` skips a position (elision)

### Object Pattern (Destructuring)

```lua
Pattern::Object(ObjectPattern)
```

```rust
struct ObjectPattern<'arena> {
    properties: &'arena [ObjectPatternProperty],
    rest: Option<Ident>,
    span: Span,
}

struct ObjectPatternProperty<'arena> {
    key: Ident,
    computed_key: Option<Expression>,
    value: Option<Pattern>,      // nested pattern (None = key used as binding)
    default: Option<Expression>,
    span: Span,
}
```

**Syntax**: `const { x, y: renamed, z = 42, ...rest } = point`

- `key` with no `value`: shorthand binding (`{ x }` binds `x`)
- `key` with `value` pattern: rename/nested destructuring (`{ y: renamed }`)
- `computed_key`: computed property name (`{ [expr]: binding }`)
- `rest`: rest element captures remaining properties

### Wildcard Pattern

```lua
Pattern::Wildcard(Span)
```

**Syntax**: `_`

Matches anything without binding a name. Used as the catch-all arm in match expressions.

### Or Pattern

```lua
Pattern::Or(OrPattern)
```

```rust
struct OrPattern<'arena> {
    alternatives: &'arena [Pattern],
    span: Span,
}
```

**Syntax**: `1 | 2 | 3 => ...`

Matches if any of the alternative patterns match.

### Template Pattern

```lua
Pattern::Template(TemplatePattern)
```

```rust
struct TemplatePattern<'arena> {
    parts: &'arena [TemplatePatternPart],
    span: Span,
}

enum TemplatePatternPart {
    String(String),
    Capture(Ident),
}
```

Matches against template-like string patterns with captures.

### Pattern Utilities

`Pattern` implements:

- `span()`: extracts the `Span` from any variant
- `node_id()`: returns `Some(StringId)` for `Identifier` patterns, `None` otherwise

---

## Types

Type annotations use the `Type<'arena>` struct. The full type system is documented in companion specs; a brief listing is included here for completeness.

```rust
struct Type<'arena> {
    kind: TypeKind,
    span: Span,
}
```

### TypeKind Variants

| Variant | Description |
| ------- | ----------- |
| `Primitive(PrimitiveType)` | `nil`, `boolean`, `number`, `integer`, `string`, `unknown`, `never`, `void`, `table`, `coroutine`, `thread` |
| `Reference(TypeReference)` | Named type with optional type arguments |
| `Union([Type])` | `A \| B` |
| `Intersection([Type])` | `A & B` |
| `Object(ObjectType)` | `{ x: number, y: string }` inline object type |
| `Array(&Type)` | `T[]` |
| `Tuple([Type])` | `[A, B, C]` |
| `Function(FunctionType)` | `(params) -> ReturnType` |
| `Literal(Literal)` | Literal types (`"hello"`, `42`, `true`) |
| `TypeQuery(&Expression)` | `typeof expr` |
| `KeyOf(&Type)` | `keyof T` |
| `IndexAccess(&Type, &Type)` | `T[K]` indexed access |
| `Conditional(ConditionalType)` | `T extends U ? A : B` |
| `Mapped(MappedType)` | `{ [K in keyof T]: ... }` |
| `TemplateLiteral(TemplateLiteralType)` | Template literal types |
| `Nullable(&Type)` | `T?` |
| `Parenthesized(&Type)` | `(T)` |
| `Infer(Ident)` | `infer R` in conditional types |
| `TypePredicate(TypePredicate)` | `x is T` type guard return |
| `Variadic(&Type)` | `...T[]` variadic return type |
| `Namespace(Vec<String>)` | File-based namespace type reference |

For complete type system details, see [Type Primitives](type-primitives.md) and [Type Advanced](type-advanced.md).

---

## Supporting Types

### TypeParameter

```rust
struct TypeParameter<'arena> {
    name: Ident,
    constraint: Option<&'arena Type>,   // extends clause
    default: Option<&'arena Type>,      // default type
    span: Span,
}
```

**Syntax**: `<T extends Comparable = string>`

### Parameter

```rust
struct Parameter<'arena> {
    pattern: Pattern,
    type_annotation: Option<Type>,
    default: Option<Expression>,
    is_rest: bool,
    is_optional: bool,
    span: Span,
}
```

**Syntax examples**:

```lua
name: string                  -- typed parameter
{ x, y }: Point               -- destructured parameter
...args: number[]             -- rest parameter (is_rest = true)
callback?: () -> void         -- optional parameter (is_optional = true)
count: number = 0             -- parameter with default
```

### Decorator

```rust
struct Decorator<'arena> {
    expression: DecoratorExpression,
    span: Span,
}

enum DecoratorExpression<'arena> {
    Identifier(Ident),
    Call { callee: &'arena DecoratorExpression, arguments: &'arena [Expression], span: Span },
    Member { object: &'arena DecoratorExpression, property: Ident, span: Span },
}
```

**Syntax**:

```lua
@injectable
@log("verbose")
@Reflect.metadata("key", "value")
```

`[NOTE: BEHAVIOR]` Avoid naming decorators `sealed`, `deprecated`, etc. -- the standard library exports these names globally, causing conflicts.

`[NOTE: BEHAVIOR]` Decorator expressions cannot use `function NS.method(target)` syntax. Use table literal syntax instead: `{ mark = function(target) ... end }`.

---

## Cross-References

- [Lexer and Tokens](lexer-and-tokens.md) -- `StringId`, `Span`, `TokenKind`, string interning
- [Parser](parser.md) -- produces these AST nodes from token streams
- [Type Primitives](type-primitives.md) -- `PrimitiveType` and `TypeKind` details
- [Type Advanced](type-advanced.md) -- conditional types, mapped types, template literal types
- [Classes](../features/classes.md) -- `ClassDeclaration` semantics and codegen
- [Modules](../features/modules.md) -- import/export resolution, re-exports, type-only imports
- [Enums](../features/enums.md) -- simple and rich enum semantics
- [Pattern Matching](../features/pattern-matching.md) -- pattern variants and match expression semantics
