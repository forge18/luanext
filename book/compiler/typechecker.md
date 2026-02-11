# Type Checker Architecture

The LuaNext type checker implements a multi-pass TypeScript-inspired type system for Lua, featuring structural subtyping, generic types, type narrowing, and incremental compilation support.

## Table of Contents

1. [Type Representation](#type-representation)
2. [Type Inference Algorithm](#type-inference-algorithm)
3. [Constraint Solving](#constraint-solving)
4. [Generic Instantiation](#generic-instantiation)
5. [Subtyping Rules](#subtyping-rules)
6. [Type Narrowing](#type-narrowing)
7. [Cross-File Type Resolution](#cross-file-type-resolution)
8. [How to Add New Type Kinds](#how-to-add-new-type-kinds)
9. [Type Serialization for Caching](#type-serialization-for-caching)

---

## Type Representation

### Type Hierarchy (`Type<'arena>`)

All types use arena allocation via `bumpalo::Bump` for efficient memory management. The core type structure is defined in `luanext-parser/src/ast/types.rs`:

```rust
pub struct Type<'arena> {
    pub kind: TypeKind<'arena>,
    pub span: Span,
}
```

### 21 Type Kinds

The `TypeKind<'arena>` enum represents all possible types in the system:

#### Primitive Types
- **Primitive(PrimitiveType)**: Built-in types (nil, boolean, number, integer, string, unknown, never, void, table, coroutine, thread)
- **Literal(Literal)**: Literal types for exact values (nil, boolean, number, string literals)

#### Structural Types
- **Object(ObjectType)**: Structural object types with properties, methods, and index signatures
- **Array(&'arena Type)**: Homogeneous array types `T[]`
- **Tuple(&'arena [Type])**: Fixed-length heterogeneous tuples `[T1, T2, T3]`
- **Function(FunctionType)**: Function signatures with parameters, return type, generic type parameters, and optional throws clause

#### Composite Types
- **Union(&'arena [Type])**: Union types `A | B | C`
- **Intersection(&'arena [Type])**: Intersection types `A & B & C`
- **Nullable(&'arena Type)**: Shorthand for `T | nil`

#### Reference Types
- **Reference(TypeReference)**: Named type references with optional type arguments (`Array<T>`, `Map<K, V>`)

#### Advanced Types
- **KeyOf(&'arena Type)**: `keyof T` - extracts property keys as union of literal types
- **IndexAccess(&'arena Type, &'arena Type)**: `T[K]` - indexed access types
- **Conditional(ConditionalType)**: `T extends U ? X : Y` - conditional types
- **Mapped(MappedType)**: `{ [K in keyof T]: V }` - mapped types with optional/readonly modifiers
- **TemplateLiteral(TemplateLiteralType)**: Template literal types with interpolation
- **TypeQuery(&'arena Expression)**: `typeof expr` - query type of expression
- **TypePredicate(TypePredicate)**: `x is T` - type guard predicates for narrowing
- **Infer(Ident)**: `infer R` - captures types in conditional type branches
- **Variadic(&'arena Type)**: `...T[]` - variadic return types for multiple returns
- **Namespace(Vec<String>)**: File-based namespace types for module imports
- **Parenthesized(&'arena Type)**: Parenthesized types for precedence

### Type Environment

The `TypeEnvironment<'arena>` manages type aliases, interfaces, and generic types:

**Location**: `crates/luanext-typechecker/src/core/type_environment.rs`

**Key features**:
- **Type aliases**: `type Foo = number`
- **Generic type aliases**: `type Container<T> = { value: T }`
- **Interfaces**: Named structural types
- **Cached primitive singletons**: Reuses `Arc<Type>` for primitives to reduce allocations
- **Utility type cache**: LRU cache for `Partial<T>`, `Pick<T, K>`, `Omit<T, K>`, etc.
- **Generic instantiation cache**: Memoizes `Container<number>` instantiations

```rust
pub struct TypeEnvironment<'arena> {
    type_aliases: FxHashMap<String, Type<'arena>>,
    generic_type_aliases: FxHashMap<String, GenericTypeAlias<'arena>>,
    interfaces: FxHashMap<String, Type<'arena>>,
    builtins: FxHashMap<String, Type<'arena>>,
    type_param_constraints: FxHashMap<String, Type<'arena>>,
    class_implements: FxHashMap<String, Vec<Type<'arena>>>,
    abstract_classes: FxHashMap<String, bool>,
    class_constructors: FxHashMap<String, &'arena [ConstructorParameter<'arena>]>,
    interface_type_params: FxHashMap<String, Vec<String>>,

    // Cached primitive types (singleton pattern)
    primitive_nil: Arc<Type<'arena>>,
    primitive_boolean: Arc<Type<'arena>>,
    primitive_number: Arc<Type<'arena>>,
    // ... (10 primitive singletons total)

    // LRU caches for expensive operations
    utility_type_cache: RefCell<FxHashMap<UtilityTypeCacheKey, Type<'arena>>>,
    generic_instantiation_cache: RefCell<FxHashMap<GenericInstantiationCacheKey, Type<'arena>>>,
}
```

**Cycle detection**: Prevents infinite recursion during type alias resolution via `resolving: RefCell<HashSet<String>>`.

---

## Type Inference Algorithm

### Two-Pass Type Checking

The type checker uses a two-pass algorithm in `check_program()`:

**Pass 1: Function Hoisting**
```rust
// Register all function declarations (hoisting)
for statement in program.statements.iter() {
    if let Some(func_decl) = extract_function_decl(statement) {
        self.register_function_signature(func_decl)?;
    }
}
```

This allows forward references: functions can be called before they appear in source order.

**Pass 2: Statement Type Checking**
```rust
// Type check all statements (including function bodies)
for statement in program.statements.iter() {
    self.check_statement(statement)?;
}
```

### Expression Type Inference

The `TypeInferenceVisitor` trait (in `visitors/inference.rs`) defines the inference interface:

**Location**: `crates/luanext-typechecker/src/visitors/inference.rs`

**Key methods**:
- `infer_expression()`: Main entry point for expression type inference
- `infer_binary_op()`: Binary operations (`+`, `==`, `and`, etc.)
- `infer_unary_op()`: Unary operations (`not`, `-`, `#`)
- `infer_call()`: Function calls with argument checking
- `infer_method()`: Method calls on objects
- `infer_member()`: Member access (`.field`)
- `infer_index()`: Index access (`[key]`)

**Implementation pattern**:
```rust
impl TypeInferenceVisitor for TypeInferrer {
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type, TypeCheckError> {
        match &expr.kind {
            ExpressionKind::Literal(lit) => {
                // Literal types: 42 has type `42` (not just `number`)
                Ok(Type::new(TypeKind::Literal(lit.clone()), span))
            }

            ExpressionKind::Identifier(name) => {
                // Check narrowing context first (control flow refinement)
                if let Some(narrowed) = self.narrowing_context.get_narrowed_type(*name) {
                    return Ok(narrowed.clone());
                }
                // Fall back to symbol table
                self.symbol_table.lookup(&name_str)?.typ
            }

            ExpressionKind::Binary(op, left, right) => {
                let left_type = self.infer_expression(left)?;
                let right_type = self.infer_expression(right)?;
                self.infer_binary_op(op, &left_type, &right_type, span)
            }

            // ... (20+ expression kinds handled)
        }
    }
}
```

### Symbol Table

The `SymbolTable<'arena>` tracks variables and their types:

**Location**: `crates/luanext-typechecker/src/utils/symbol_table.rs`

**Features**:
- **Scoped symbol resolution**: Nested scopes with shadowing
- **Symbol kinds**: Variable, Function, TypeAlias, Interface, Enum, Class
- **Mutability tracking**: `is_mutable` flag for const checking
- **Export tracking**: `is_exported` flag for module exports

```rust
pub struct Symbol<'arena> {
    pub name: String,
    pub kind: SymbolKind,
    pub typ: Type<'arena>,
    pub span: Span,
    pub is_mutable: bool,
    pub is_exported: bool,
}
```

---

## Constraint Solving

### Type Compatibility

The `TypeCompatibility` checker implements structural subtyping with variance rules.

**Location**: `crates/luanext-typechecker/src/core/type_compat.rs`

**Core algorithm**:
```rust
pub fn is_assignable(source: &Type, target: &Type) -> bool {
    is_assignable_recursive(source, target, &mut visited_set)
}
```

**Visited set**: Prevents infinite loops when checking recursive types.

### Special Type Rules

**Unknown type**:
```rust
// Unknown is assignable to/from anything (escape hatch)
if matches!(source, TypeKind::Primitive(PrimitiveType::Unknown)) ||
   matches!(target, TypeKind::Primitive(PrimitiveType::Unknown)) {
    return true;
}
```

**Never type**:
```rust
// Never is assignable to anything (bottom type)
if matches!(source, TypeKind::Primitive(PrimitiveType::Never)) {
    return true;
}
// Nothing is assignable to Never
if matches!(target, TypeKind::Primitive(PrimitiveType::Never)) {
    return false;
}
```

**Integer subtyping**:
```rust
// Integer is assignable to number (widening)
(PrimitiveType::Integer, PrimitiveType::Number) => true,
```

### Union and Intersection Types

**Union**: Source is assignable to union if assignable to **any** member
```rust
(_, TypeKind::Union(targets)) => {
    targets.iter().any(|t| is_assignable(source, t))
}
```

**Intersection**: Source is assignable to intersection if assignable to **all** members
```rust
(_, TypeKind::Intersection(targets)) => {
    targets.iter().all(|t| is_assignable(source, t))
}
```

### Object Type Structural Compatibility

For object types, check that target properties exist in source:

```rust
fn is_object_assignable(source: &ObjectType, target: &ObjectType) -> bool {
    for t_member in target.members.iter() {
        match t_member {
            ObjectTypeMember::Property(t_prop) => {
                // Find matching property in source
                let found = source.members.iter().any(|s_member| {
                    if let ObjectTypeMember::Property(s_prop) = s_member {
                        s_prop.name == t_prop.name &&
                        is_assignable(&s_prop.type_annotation, &t_prop.type_annotation)
                    } else {
                        false
                    }
                });
                if !found && !t_prop.is_optional {
                    return false;
                }
            }
            // ... (method and index signature checking)
        }
    }
    true
}
```

### Type Alias Resolution

When comparing type references, the checker resolves aliases using `TypeEnvironment`:

```rust
pub fn is_assignable_with_env(
    source: &Type,
    target: &Type,
    type_env: &TypeEnvironment,
    interner: &StringInterner,
) -> bool {
    match (&source.kind, &target.kind) {
        (TypeKind::Reference(s_ref), TypeKind::Reference(t_ref)) => {
            // Resolve both to their underlying types
            let resolved_source = type_env.lookup_type_alias(&s_name);
            let resolved_target = type_env.lookup_type_alias(&t_name);
            // Check structural compatibility
            is_assignable(resolved_source, resolved_target)
        }
        // ...
    }
}
```

### Type Relation Cache

To avoid redundant checks, the typechecker uses an LRU cache:

**Location**: `crates/luanext-typechecker/src/type_relations.rs`

```rust
pub struct TypeRelationCache {
    cache: LruCache<(usize, usize), bool>,
}

impl TypeRelationCache {
    pub fn get(&self, source: &Type, target: &Type) -> Option<bool> {
        let key = (source as *const Type as usize, target as *const Type as usize);
        self.cache.peek(&key).copied()
    }
}
```

---

## Generic Instantiation

### Generic Type Parameters

Generic types are defined with type parameters:

```typescript
type Container<T> = { value: T }
class List<T> { ... }
function identity<T>(x: T): T { return x; }
```

### Substitution Algorithm

The `instantiate_type()` function substitutes type parameters with concrete types.

**Location**: `crates/luanext-typechecker/src/types/generics.rs`

**Algorithm**:
1. Build substitution map: `{ T -> number, U -> string }`
2. Recursively traverse type structure
3. Replace type references matching parameters

```rust
pub fn instantiate_type<'arena>(
    arena: &'arena bumpalo::Bump,
    typ: &Type<'arena>,
    type_params: &[TypeParameter<'arena>],
    type_args: &[Type<'arena>],
) -> Result<Type<'arena>, String> {
    // Build substitution map
    let mut substitutions: FxHashMap<StringId, Type<'arena>> = FxHashMap::default();
    for (param, arg) in type_params.iter().zip(type_args.iter()) {
        substitutions.insert(param.name.node, arg.clone());
    }

    substitute_type(arena, typ, &substitutions)
}

fn substitute_type<'arena>(
    arena: &'arena bumpalo::Bump,
    typ: &Type<'arena>,
    substitutions: &FxHashMap<StringId, Type<'arena>>,
) -> Result<Type<'arena>, String> {
    match &typ.kind {
        TypeKind::Reference(type_ref) => {
            // Check if this is a type parameter
            if let Some(substituted) = substitutions.get(&type_ref.name.node) {
                return Ok(substituted.clone());
            }
            // Not a parameter - recursively substitute type arguments
            let new_args = type_ref.type_arguments.map(|args| {
                args.iter()
                    .map(|arg| substitute_type(arena, arg, substitutions))
                    .collect::<Result<Vec<_>, _>>()
            });
            // Reconstruct TypeReference with substituted args
            Ok(Type::new(TypeKind::Reference(new_type_ref), typ.span))
        }

        TypeKind::Object(obj) => {
            // Substitute in all property types
            let new_members = obj.members.iter()
                .map(|member| substitute_in_member(arena, member, substitutions))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::new(TypeKind::Object(new_obj), typ.span))
        }

        // ... (handle all type kinds recursively)
    }
}
```

### Generic Type Alias Instantiation

The `TypeEnvironment` caches generic instantiations:

```rust
pub fn instantiate_generic_type(
    &self,
    arena: &'arena bumpalo::Bump,
    name: &str,
    type_args: &[Type<'arena>],
    span: Span,
) -> Result<Type<'arena>, String> {
    // Check cache first
    let cache_key = GenericInstantiationCacheKey {
        name: name.to_string(),
        type_args_hash: compute_type_args_fingerprint(type_args),
    };

    if let Some(cached) = self.generic_instantiation_cache.borrow().get(&cache_key) {
        return Ok(cached.clone());
    }

    // Instantiate and cache
    let generic_alias = self.generic_type_aliases.get(name)?;
    let instantiated = instantiate_type(
        arena,
        &generic_alias.typ,
        &generic_alias.type_parameters,
        type_args,
    )?;

    self.generic_instantiation_cache.borrow_mut().insert(cache_key, instantiated.clone());
    Ok(instantiated)
}
```

### Type Parameter Constraints

Type parameters can have constraints:

```typescript
function sorted<T extends Comparable>(arr: T[]): T[] { ... }
```

Constraints are stored in `TypeEnvironment::type_param_constraints` and checked during instantiation.

---

## Subtyping Rules

### Variance in Function Types

Function types exhibit **contravariance** in parameters and **covariance** in return types.

**Location**: `crates/luanext-typechecker/src/core/type_compat.rs:376-400`

```rust
fn is_function_assignable(
    source: &FunctionType,
    target: &FunctionType,
    visited: &mut HashSet<(usize, usize)>,
) -> bool {
    // Parameters are contravariant: target params must be assignable to source params
    // This means the source function can accept MORE specific types than target expects
    for (s_param, t_param) in source.parameters.iter().zip(target.parameters.iter()) {
        if let (Some(s_type), Some(t_type)) = (&s_param.type_annotation, &t_param.type_annotation) {
            if !is_assignable_recursive(t_type, s_type, visited) {  // Note: reversed!
                return false;
            }
        }
    }

    // Return type is covariant: source return must be assignable to target return
    is_assignable_recursive(source.return_type, target.return_type, visited)
}
```

**Example**:
```typescript
type Handler = (x: Animal) => Dog;
let handler: Handler;

// OK: accepts more specific input (contravariant)
handler = (x: Dog) => new Dog();  // Dog <: Animal

// OK: returns more specific output (covariant)
handler = (x: Animal) => new Puppy();  // Puppy <: Dog

// ERROR: parameter type is less specific
handler = (x: Object) => new Dog();  // Object >: Animal (not allowed)

// ERROR: return type is less specific
handler = (x: Animal) => new Animal();  // Animal >: Dog (not allowed)
```

### Method Override Checking

When a class method overrides a parent method, variance rules apply:

**Location**: `crates/luanext-typechecker/src/phases/validation_phase.rs:303-457`

```rust
// Parameters are contravariant: parent type must be assignable to child type
// (child can accept a more specific type than parent)
if !TypeCompatibility::is_assignable(&parent_param_type, &child_param_type) {
    return Err(TypeCheckError::new(
        format!(
            "Override of '{}' has incompatible parameter type. \
             Expected '{}' but got '{}'",
            method_name, parent_type_str, child_type_str
        ),
        child_param.span,
    ));
}

// Return type is covariant: child return must be assignable to parent return
if !TypeCompatibility::is_assignable(&child_return, &parent_return) {
    return Err(TypeCheckError::new(
        format!(
            "Override of '{}' has incompatible return type. \
             Expected '{}' but got '{}'",
            method_name, parent_return_str, child_return_str
        ),
        child_method.span,
    ));
}
```

### Literal Types and Widening

Literal types are subtypes of their primitive types:

```rust
fn is_literal_assignable_to_primitive(lit: &Literal, prim: PrimitiveType) -> bool {
    matches!(
        (lit, prim),
        (Literal::Number(_), PrimitiveType::Number) |
        (Literal::String(_), PrimitiveType::String) |
        (Literal::Boolean(_), PrimitiveType::Boolean) |
        (Literal::Nil, PrimitiveType::Nil)
    )
}
```

**Example**:
```typescript
let x: 42 = 42;        // OK: literal type
let y: number = x;     // OK: 42 <: number (widening)
let z: 42 = y;         // ERROR: number is not assignable to 42
```

### Structural Subtyping for Objects

Objects use structural subtyping (duck typing):

```typescript
interface Person { name: string; age: number; }
interface Named { name: string; }

let person: Person = { name: "Alice", age: 30 };
let named: Named = person;  // OK: Person has all properties of Named
```

**Excess property checking** only applies at object literal creation sites:

```typescript
let named: Named = { name: "Alice", age: 30 };  // ERROR: excess property 'age'
```

---

## Type Narrowing

Type narrowing refines variable types based on control flow analysis.

**Location**: `crates/luanext-typechecker/src/visitors/narrowing.rs`

### Narrowing Context

The `NarrowingContext` tracks refined types within a scope:

```rust
pub struct NarrowingContext<'arena> {
    narrowed_types: FxHashMap<StringId, Type<'arena>>,
}

impl NarrowingContext {
    pub fn get_narrowed_type(&self, name: StringId) -> Option<&Type<'arena>>;
    pub fn set_narrowed_type(&mut self, name: StringId, typ: Type<'arena>);
    pub fn merge(then_ctx: &Self, else_ctx: &Self) -> Self;
}
```

### typeof Guards

Type narrowing from `typeof` checks:

```typescript
function process(x: number | string) {
    if (typeof x == "string") {
        // x is narrowed to string here
        print(x.upper())
    } else {
        // x is narrowed to number here
        print(x + 1)
    }
}
```

**Implementation**:
```rust
fn narrow_type_from_condition(
    condition: &Expression,
    base_ctx: &NarrowingContext,
    original_types: &FxHashMap<StringId, Type>,
) -> (NarrowingContext, NarrowingContext) {
    match &condition.kind {
        ExpressionKind::Binary(BinaryOp::Equal, left, right) => {
            if let Some((var_name, type_name)) = extract_typeof_check(left, right) {
                if let Some(narrowed_type) = typeof_string_to_type(&type_name) {
                    // Then branch: x has the checked type
                    then_ctx.set_narrowed_type(var_name, narrowed_type);

                    // Else branch: exclude the checked type
                    if let Some(original) = original_types.get(&var_name) {
                        if let Some(else_type) = exclude_type(original, &narrowed_type) {
                            else_ctx.set_narrowed_type(var_name, else_type);
                        }
                    }
                }
            }
        }
        // ... (other narrowing patterns)
    }
    (then_ctx, else_ctx)
}
```

### Nil Narrowing

Checking for `nil` refines nullable types:

```typescript
function greet(name: string?) {
    if (name != nil) {
        // name is narrowed to string (non-nil)
        print("Hello, " .. name)
    } else {
        // name is nil
        print("Hello, stranger")
    }
}
```

**Implementation**:
```rust
if let Some((var_name, is_nil)) = extract_nil_check(left, right) {
    if is_nil {
        // x == nil
        then_ctx.set_narrowed_type(var_name, Type::Primitive(PrimitiveType::Nil));

        // x != nil (remove nil from union)
        if let Some(non_nil) = remove_nil_from_type(original_type) {
            else_ctx.set_narrowed_type(var_name, non_nil);
        }
    }
}
```

### Type Predicates

User-defined type guards using `x is T` syntax:

```typescript
function isString(x: unknown): x is string {
    return typeof x == "string"
}

function process(x: unknown) {
    if (isString(x)) {
        // x is narrowed to string
        print(x.upper())
    }
}
```

### Branch Merging

At branch join points, the narrowing context merges:

```rust
impl NarrowingContext {
    pub fn merge(then_ctx: &Self, else_ctx: &Self) -> Self {
        let mut merged = NarrowingContext::new();

        // Only keep types that are the same in both branches
        for (name, then_type) in &then_ctx.narrowed_types {
            if let Some(else_type) = else_ctx.narrowed_types.get(name) {
                if types_equal(then_type, else_type) {
                    merged.narrowed_types.insert(*name, then_type.clone());
                }
                // TODO: Create union type for divergent branches
            }
        }

        merged
    }
}
```

---

## Cross-File Type Resolution

The module system enables importing types and values across files.

### Module Registry

The `ModuleRegistry` stores compiled modules and their exports.

**Location**: `crates/luanext-typechecker/src/module_resolver/registry.rs`

```rust
pub struct ModuleRegistry {
    modules: RwLock<FxHashMap<ModuleId, CompiledModule>>,
}

pub struct CompiledModule {
    pub id: ModuleId,
    pub exports: ModuleExports,
    pub symbol_table: Arc<SymbolTable<'static>>,
    pub status: ModuleStatus,
}

pub struct ModuleExports {
    pub named: IndexMap<String, ExportedSymbol>,
    pub default: Option<ExportedSymbol>,
}

pub struct ExportedSymbol {
    pub symbol: Symbol<'static>,
    pub is_type_only: bool,
}
```

**Note**: Uses `'static` lifetime because registry outlives any single arena. Symbols are converted from `Symbol<'arena>` to `Symbol<'static>` via `unsafe transmute` in `symbol_to_static()`.

### Module Resolution

The `ModuleResolver` maps import paths to module IDs:

**Location**: `crates/luanext-typechecker/src/module_resolver/mod.rs`

```rust
pub struct ModuleResolver {
    root_path: PathBuf,
    module_paths: RwLock<FxHashMap<ModuleId, PathBuf>>,
}

impl ModuleResolver {
    pub fn resolve_import(&self, from_path: &Path, import_path: &str) -> Result<ModuleId>;
}
```

**Resolution algorithm**:
1. Relative imports (`./foo`, `../bar`) resolve relative to importing file
2. Absolute imports (`foo/bar`) resolve relative to project root
3. Check for `.lnx` file at resolved path
4. Return `ModuleId` (canonical path hash)

### Import Type Checking

When checking an import statement:

```rust
fn check_import_declaration(
    &mut self,
    import_decl: &ImportDeclaration,
) -> Result<(), TypeCheckError> {
    // Resolve module path
    let module_id = self.module_resolver.resolve_import(
        &self.current_module_id,
        &import_decl.module_specifier,
    )?;

    // Get exports from registry
    let exports = self.module_registry.get_exports(&module_id)?;

    // Check named imports
    for import in &import_decl.named_imports {
        let export_symbol = exports.get_named(&import.name)
            .ok_or_else(|| TypeCheckError::new(
                format!("Module '{}' has no export '{}'", module_id, import.name),
                import.span,
            ))?;

        // Register in local symbol table
        self.symbol_table.insert(
            import.alias.unwrap_or(import.name),
            export_symbol.symbol.clone(),
        );
    }

    Ok(())
}
```

### Dependency Graph

The dependency graph detects circular imports:

**Location**: `crates/luanext-typechecker/src/module_resolver/dependency_graph.rs`

```rust
pub struct DependencyGraph {
    adjacency: FxHashMap<ModuleId, Vec<ModuleId>>,
}

impl DependencyGraph {
    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId);
    pub fn has_cycle(&self) -> bool;
    pub fn topological_sort(&self) -> Result<Vec<ModuleId>, CycleError>;
}
```

**Cycle detection** uses depth-first search with a visited set.

### Module Status Tracking

Modules progress through stages:

```rust
pub enum ModuleStatus {
    Parsed,              // AST generated, not type-checked
    ExportsExtracted,    // Exports extracted but body not checked
    TypeChecked,         // Fully type-checked
}
```

**Export extraction** happens before full type checking to resolve import dependencies:

```rust
// Extract exports without checking bodies
pub fn extract_exports(&self, program: &Program) -> ModuleExports {
    let mut exports = ModuleExports::new();
    for statement in program.statements {
        if let Statement::Export(export_decl) = statement {
            match &export_decl.kind {
                ExportKind::Named(named) => {
                    let symbol = self.symbol_table.lookup(&named.name)?;
                    exports.add_named(named.alias.unwrap_or(named.name), symbol);
                }
                ExportKind::Default(expr) => {
                    let typ = self.infer_expression_shallow(expr)?;
                    exports.set_default(typ);
                }
            }
        }
    }
    exports
}
```

---

## How to Add New Type Kinds

To add a new type kind to the system, follow these steps:

### 1. Add to TypeKind Enum

**File**: `crates/luanext-parser/src/ast/types.rs`

```rust
#[derive(Debug, Clone, Serialize)]
pub enum TypeKind<'arena> {
    // ... existing kinds ...

    /// New type kind
    #[serde(borrow)]
    MyNewType(MyNewTypeData<'arena>),
}

#[derive(Debug, Clone, Serialize)]
pub struct MyNewTypeData<'arena> {
    #[serde(borrow)]
    pub field1: &'arena Type<'arena>,
    pub field2: String,
    pub span: Span,
}
```

**Important**: Use `#[serde(borrow)]` for arena references.

### 2. Update Type Compatibility

**File**: `crates/luanext-typechecker/src/core/type_compat.rs`

Add matching logic in `is_assignable_recursive()`:

```rust
fn is_assignable_recursive(source: &Type, target: &Type, visited: &mut HashSet) -> bool {
    match (&source.kind, &target.kind) {
        // ... existing cases ...

        (TypeKind::MyNewType(s_data), TypeKind::MyNewType(t_data)) => {
            // Define your subtyping rule
            is_assignable_recursive(s_data.field1, t_data.field1, visited) &&
            s_data.field2 == t_data.field2
        }

        _ => false,
    }
}
```

### 3. Add Parser Support

**File**: `crates/luanext-parser/src/parser.rs`

Add parsing logic:

```rust
fn parse_type(&mut self) -> ParseResult<Type<'arena>> {
    match self.current_token() {
        // ... existing cases ...

        Token::MyNewKeyword => {
            self.advance();  // consume keyword
            let field1 = self.parse_type()?;
            let field2 = self.parse_identifier()?;
            Ok(Type::new(
                TypeKind::MyNewType(arena.alloc(MyNewTypeData {
                    field1: arena.alloc(field1),
                    field2,
                    span,
                })),
                span,
            ))
        }

        _ => Err(ParseError::ExpectedType(self.current_span()))
    }
}
```

### 4. Update Generic Instantiation

**File**: `crates/luanext-typechecker/src/types/generics.rs`

Add substitution logic:

```rust
fn substitute_type(typ: &Type, substitutions: &FxHashMap<StringId, Type>) -> Result<Type> {
    match &typ.kind {
        // ... existing cases ...

        TypeKind::MyNewType(data) => {
            let new_field1 = substitute_type(data.field1, substitutions)?;
            Ok(Type::new(
                TypeKind::MyNewType(arena.alloc(MyNewTypeData {
                    field1: arena.alloc(new_field1),
                    field2: data.field2.clone(),
                    span: data.span,
                })),
                typ.span,
            ))
        }
    }
}
```

### 5. Add Type Inference Support

**File**: `crates/luanext-typechecker/src/visitors/inference.rs`

If expressions can have your new type:

```rust
impl TypeInferenceVisitor for TypeInferrer {
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type> {
        match &expr.kind {
            // ... existing cases ...

            ExpressionKind::MyNewExpr(data) => {
                let inner_type = self.infer_expression(data.inner)?;
                Ok(Type::new(
                    TypeKind::MyNewType(arena.alloc(MyNewTypeData {
                        field1: arena.alloc(inner_type),
                        field2: data.field2.clone(),
                        span: expr.span,
                    })),
                    expr.span,
                ))
            }
        }
    }
}
```

### 6. Update Serialization

**File**: `crates/luanext-core/src/cache/serializable_types.rs`

Add serializable equivalent:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableTypeKind {
    // ... existing variants ...
    MyNewType(SerializableMyNewType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMyNewType {
    pub field1: Box<SerializableType>,
    pub field2: String,
    pub span: Span,
}

impl SerializableTypeKind {
    fn from_type_kind(kind: &TypeKind, interner: &StringInterner) -> Self {
        match kind {
            // ... existing cases ...
            TypeKind::MyNewType(data) => {
                SerializableTypeKind::MyNewType(SerializableMyNewType {
                    field1: Box::new(SerializableType::from_type(data.field1, interner)),
                    field2: data.field2.clone(),
                    span: data.span,
                })
            }
        }
    }

    fn to_type_kind(&self, interner: &StringInterner) -> TypeKind<'static> {
        match self {
            // ... existing cases ...
            SerializableTypeKind::MyNewType(data) => {
                TypeKind::MyNewType(Box::leak(Box::new(MyNewTypeData {
                    field1: Box::leak(Box::new(data.field1.to_type(interner))),
                    field2: data.field2.clone(),
                    span: data.span,
                })))
            }
        }
    }
}
```

### 7. Add Tests

**File**: `crates/luanext-typechecker/tests/type_checking_tests.rs`

```rust
#[test]
fn test_my_new_type() {
    let source = r#"
        type MyType = mynewtype<number, "foo">
        let x: MyType = ...
    "#;

    let result = type_check(source);
    assert!(result.is_ok());
}

#[test]
fn test_my_new_type_assignability() {
    let source = r#"
        type A = mynewtype<number, "foo">
        type B = mynewtype<number, "bar">
        let a: A = ...
        let b: B = a  -- should fail
    "#;

    let result = type_check(source);
    assert!(result.is_err());
}
```

---

## Type Serialization for Caching

To support incremental compilation, types must be serialized to disk.

### Challenge: Arena Lifetimes

Arena-allocated types (`Type<'arena>`) use `&'arena` references, which cannot be serialized:

```rust
// Cannot derive Deserialize because of lifetime
pub struct Type<'arena> {
    pub kind: TypeKind<'arena>,  // Contains &'arena references
    pub span: Span,
}
```

### Solution: Owned Type Hierarchy

Create owned equivalents using `Vec<T>` and `Box<T>`:

**File**: `crates/luanext-core/src/cache/serializable_types.rs`

```rust
/// Owned serializable equivalent of Type<'arena>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableType {
    pub kind: SerializableTypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableTypeKind {
    Primitive(PrimitiveType),
    Reference(SerializableTypeReference),
    Union(Vec<SerializableType>),            // Vec instead of &'arena [Type]
    Array(Box<SerializableType>),            // Box instead of &'arena Type
    Object(SerializableObjectType),
    Function(SerializableFunctionType),
    // ... (11 total variants - subset of full TypeKind)
    Unknown,  // Fallback for complex types (Conditional, Mapped, etc.)
}
```

### Conversion Functions

**From arena type to serializable**:
```rust
impl SerializableType {
    pub fn from_type(ty: &Type<'_>, interner: &StringInterner) -> Self {
        SerializableType {
            kind: SerializableTypeKind::from_type_kind(&ty.kind, interner),
            span: ty.span,
        }
    }
}

impl SerializableTypeKind {
    fn from_type_kind(kind: &TypeKind<'_>, interner: &StringInterner) -> Self {
        match kind {
            TypeKind::Union(members) => {
                // Convert &'arena [Type] to Vec<SerializableType>
                SerializableTypeKind::Union(
                    members.iter()
                        .map(|t| SerializableType::from_type(t, interner))
                        .collect()
                )
            }

            TypeKind::Reference(r) => {
                // Convert StringId to String
                SerializableTypeKind::Reference(SerializableTypeReference {
                    name: interner.resolve(r.name.node),
                    type_arguments: r.type_arguments.map(|args| {
                        args.iter()
                            .map(|t| SerializableType::from_type(t, interner))
                            .collect()
                    }),
                    span: r.span,
                })
            }

            // Complex types fall back to Unknown
            TypeKind::Conditional(_) | TypeKind::Mapped(_) => {
                SerializableTypeKind::Unknown
            }

            // ... (handle all 21 type kinds)
        }
    }
}
```

**From serializable to arena type**:
```rust
impl SerializableType {
    /// Convert back to Type<'static>
    /// Uses Box::leak to create 'static references
    pub fn to_type(&self, interner: &StringInterner) -> Type<'static> {
        Type {
            kind: self.kind.to_type_kind(interner),
            span: self.span,
        }
    }
}

impl SerializableTypeKind {
    fn to_type_kind(&self, interner: &StringInterner) -> TypeKind<'static> {
        match self {
            SerializableTypeKind::Union(members) => {
                // Convert Vec<SerializableType> to &'static [Type]
                let types: Vec<Type<'static>> = members.iter()
                    .map(|t| t.to_type(interner))
                    .collect();
                TypeKind::Union(Box::leak(types.into_boxed_slice()))
            }

            SerializableTypeKind::Reference(r) => {
                // Convert String to StringId
                let name_id = interner.get_or_intern(&r.name);
                TypeKind::Reference(Box::leak(Box::new(TypeReference {
                    name: Spanned::new(name_id, r.span),
                    type_arguments: r.type_arguments.as_ref().map(|args| {
                        let types: Vec<Type<'static>> = args.iter()
                            .map(|t| t.to_type(interner))
                            .collect();
                        Box::leak(types.into_boxed_slice())
                    }),
                    span: r.span,
                })))
            }

            // ... (handle all variants)
        }
    }
}
```

### Memory Management

The `to_type()` function uses `Box::leak()` to create `'static` references. This is acceptable because:

1. **Small memory footprint**: Only export types are serialized (not entire AST)
2. **Session lifetime**: Leaked memory lives for the compilation session
3. **CLI mode**: Process exits after compilation, OS reclaims memory
4. **LSP mode**: Should use dedicated arena pool (future enhancement)

### Module Export Serialization

The cache stores exports for each module:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModuleExports {
    pub named: Vec<(String, SerializableExportedSymbol)>,
    pub default: Option<SerializableExportedSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExportedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub typ: SerializableType,
    pub span: Span,
    pub is_exported: bool,
    pub is_type_only: bool,
}
```

### Cache Structure

**File**: `crates/luanext-core/src/cache/mod.rs`

```rust
pub struct CachedModule {
    pub path: PathBuf,
    pub hash: u64,
    pub ast_hash: u64,
    pub dependencies: Vec<PathBuf>,
    pub serializable_exports: Option<SerializableModuleExports>,
}

pub struct CacheManifest {
    pub version: u32,
    pub modules: HashMap<PathBuf, CachedModule>,
}
```

**Cache invalidation**:
- File content hash changed
- Dependency hash changed
- Cache version mismatch

### Loading from Cache

```rust
pub fn load_from_cache(
    &self,
    module_id: &ModuleId,
) -> Option<ModuleExports> {
    let cached = self.cache_manifest.get(module_id)?;

    // Check if cache is valid
    if !self.is_cache_valid(&cached) {
        return None;
    }

    // Deserialize exports
    let serializable_exports = cached.serializable_exports?;
    let exports = ModuleExports::from_serializable(
        &serializable_exports,
        &self.interner,
    );

    Some(exports)
}
```

---

## Performance Optimizations

### Type Caching

1. **Primitive type singletons**: Reuse `Arc<Type>` for primitives
2. **Utility type cache**: LRU cache for `Partial<T>`, `Pick<T, K>`, etc.
3. **Generic instantiation cache**: Memoize `Container<number>` expansions
4. **Type relation cache**: LRU cache for subtype checks

### Arena Allocation

All AST and type nodes use `bumpalo::Bump` arena:
- **Fast allocation**: Bump pointer, no individual frees
- **Locality**: Better cache performance
- **Lifetime safety**: Enforced by `'arena` lifetime

### String Interning

The `StringInterner` deduplicates strings:
- **Unique IDs**: `StringId` is `Copy` and cheap to compare
- **Lookup**: O(1) hash table lookup
- **Serialization**: Round-trip via `to_strings()` / `from_strings()`

---

## Future Enhancements

1. **Better union type merging**: Create union types at branch join points instead of discarding narrowed types
2. **Exhaustiveness checking**: Verify all union cases are handled in match expressions
3. **Flow-sensitive typing**: Track mutability and reassignments more precisely
4. **Inference improvements**: Infer generic type arguments from usage context
5. **LSP arena pooling**: Reuse arenas across LSP requests to avoid memory leaks
6. **Incremental module checking**: Skip unchanged modules in multi-module projects
7. **Type-only imports**: Elide type-only imports at runtime for smaller bundles

---

## References

**Source code locations**:
- Type representation: `luanext-parser/src/ast/types.rs`
- Type checker: `luanext-typechecker/src/core/type_checker.rs`
- Type compatibility: `luanext-typechecker/src/core/type_compat.rs`
- Type environment: `luanext-typechecker/src/core/type_environment.rs`
- Type inference: `luanext-typechecker/src/visitors/inference.rs`
- Type narrowing: `luanext-typechecker/src/visitors/narrowing.rs`
- Generics: `luanext-typechecker/src/types/generics.rs`
- Utility types: `luanext-typechecker/src/types/utility_types.rs`
- Module registry: `luanext-typechecker/src/module_resolver/registry.rs`
- Serialization: `luanext-core/src/cache/serializable_types.rs`

**Key algorithms**:
- Structural subtyping with cycle detection
- Contravariant/covariant variance checking
- Control flow-based type narrowing
- Generic type substitution
- Topological module sorting

**Design patterns**:
- Arena allocation for AST and types
- Visitor pattern for type checking phases
- LRU caching for expensive operations
- Owned types for serialization
- Symbol table with scoped lookup

Agent is calibrated...
