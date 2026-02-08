//! Owned serializable type hierarchy for incremental compilation cache.
//!
//! Arena-allocated AST types (`Type<'arena>`) cannot be deserialized because they
//! use `&'arena` references. This module provides owned equivalents using `Vec<T>`
//! and `Box<T>` that can be serialized to/from disk via serde.
//!
//! Only the subset of types needed for exported symbols is represented here.
//! Complex types that rarely appear in exports (Conditional, Mapped, etc.)
//! fall back to `Unknown`.

use luanext_parser::ast::expression::Literal;
use luanext_parser::ast::statement::IndexKeyType;
use luanext_parser::ast::types::*;
use luanext_parser::ast::{Ident, Spanned};
use luanext_parser::span::Span;
use luanext_parser::string_interner::StringInterner;
use luanext_typechecker::module_resolver::{ExportedSymbol, ModuleExports};
use luanext_typechecker::{Symbol, SymbolKind};
use serde::{Deserialize, Serialize};

/// Owned serializable equivalent of `Type<'arena>`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableType {
    pub kind: SerializableTypeKind,
    pub span: Span,
}

/// Owned serializable equivalent of `TypeKind<'arena>`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableTypeKind {
    Primitive(PrimitiveType),
    Reference(SerializableTypeReference),
    Union(Vec<SerializableType>),
    Intersection(Vec<SerializableType>),
    Object(SerializableObjectType),
    Array(Box<SerializableType>),
    Tuple(Vec<SerializableType>),
    Function(SerializableFunctionType),
    Literal(SerializableLiteral),
    Nullable(Box<SerializableType>),
    Namespace(Vec<String>),
    /// Fallback for complex types we don't serialize (KeyOf, Conditional, etc.)
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTypeReference {
    pub name: String,
    pub type_arguments: Option<Vec<SerializableType>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableObjectType {
    pub members: Vec<SerializableObjectMember>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableObjectMember {
    Property(SerializableProperty),
    Method(SerializableMethod),
    Index(SerializableIndex),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProperty {
    pub is_readonly: bool,
    pub name: String,
    pub is_optional: bool,
    pub type_annotation: SerializableType,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMethod {
    pub name: String,
    pub parameters: Vec<SerializableParameter>,
    pub return_type: SerializableType,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableIndex {
    pub key_name: String,
    pub key_type: IndexKeyType,
    pub value_type: SerializableType,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableFunctionType {
    pub type_parameters: Option<Vec<SerializableTypeParameter>>,
    pub parameters: Vec<SerializableParameter>,
    pub return_type: Box<SerializableType>,
    pub throws: Option<Vec<SerializableType>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTypeParameter {
    pub name: String,
    pub constraint: Option<Box<SerializableType>>,
    pub default: Option<Box<SerializableType>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableParameter {
    pub name: String,
    pub type_annotation: Option<SerializableType>,
    pub is_rest: bool,
    pub is_optional: bool,
    pub span: Span,
}

/// Owned equivalent of `expression::Literal` (which lacks `Deserialize`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableLiteral {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(String),
}

/// Serializable equivalent of `ExportedSymbol`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExportedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub typ: SerializableType,
    pub span: Span,
    pub is_exported: bool,
    pub is_type_only: bool,
}

/// Serializable equivalent of `ModuleExports`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModuleExports {
    pub named: Vec<(String, SerializableExportedSymbol)>,
    pub default: Option<SerializableExportedSymbol>,
}

// ---------------------------------------------------------------------------
// Conversion: Type<'_> -> SerializableType
// ---------------------------------------------------------------------------

impl SerializableType {
    /// Convert from arena-allocated `Type` to owned `SerializableType`.
    pub fn from_type(ty: &Type<'_>, interner: &StringInterner) -> Self {
        SerializableType {
            kind: SerializableTypeKind::from_type_kind(&ty.kind, interner),
            span: ty.span,
        }
    }

    /// Convert back to `Type<'static>`.
    ///
    /// Uses `Box::leak` to create `&'static` references for interior types.
    /// The leaked memory is small (export types only) and lives for the
    /// compilation session. For CLI this is fine; for LSP a dedicated arena
    /// should be used in the future.
    pub fn to_type(&self, interner: &StringInterner) -> Type<'static> {
        Type {
            kind: self.kind.to_type_kind(interner),
            span: self.span,
        }
    }
}

impl SerializableTypeKind {
    fn from_type_kind(kind: &TypeKind<'_>, interner: &StringInterner) -> Self {
        match kind {
            TypeKind::Primitive(p) => SerializableTypeKind::Primitive(*p),
            TypeKind::Reference(r) => SerializableTypeKind::Reference(SerializableTypeReference {
                name: interner.resolve(r.name.node),
                type_arguments: r.type_arguments.map(|args| {
                    args.iter()
                        .map(|t| SerializableType::from_type(t, interner))
                        .collect()
                }),
                span: r.span,
            }),
            TypeKind::Union(members) => SerializableTypeKind::Union(
                members
                    .iter()
                    .map(|t| SerializableType::from_type(t, interner))
                    .collect(),
            ),
            TypeKind::Intersection(members) => SerializableTypeKind::Intersection(
                members
                    .iter()
                    .map(|t| SerializableType::from_type(t, interner))
                    .collect(),
            ),
            TypeKind::Object(obj) => {
                SerializableTypeKind::Object(SerializableObjectType::from_object(obj, interner))
            }
            TypeKind::Array(elem) => {
                SerializableTypeKind::Array(Box::new(SerializableType::from_type(elem, interner)))
            }
            TypeKind::Tuple(elems) => SerializableTypeKind::Tuple(
                elems
                    .iter()
                    .map(|t| SerializableType::from_type(t, interner))
                    .collect(),
            ),
            TypeKind::Function(func) => SerializableTypeKind::Function(
                SerializableFunctionType::from_function(func, interner),
            ),
            TypeKind::Literal(lit) => {
                SerializableTypeKind::Literal(SerializableLiteral::from_literal(lit))
            }
            TypeKind::Nullable(inner) => SerializableTypeKind::Nullable(Box::new(
                SerializableType::from_type(inner, interner),
            )),
            TypeKind::Namespace(parts) => SerializableTypeKind::Namespace(parts.clone()),
            // Complex types that are rare in exports — fall back to Unknown
            TypeKind::TypeQuery(_)
            | TypeKind::KeyOf(_)
            | TypeKind::IndexAccess(_, _)
            | TypeKind::Conditional(_)
            | TypeKind::Mapped(_)
            | TypeKind::TemplateLiteral(_)
            | TypeKind::Parenthesized(_)
            | TypeKind::Infer(_)
            | TypeKind::TypePredicate(_)
            | TypeKind::Variadic(_) => SerializableTypeKind::Unknown,
        }
    }

    fn to_type_kind(&self, interner: &StringInterner) -> TypeKind<'static> {
        match self {
            SerializableTypeKind::Primitive(p) => TypeKind::Primitive(*p),
            SerializableTypeKind::Reference(r) => {
                let string_id = interner.intern(&r.name);
                let ident = Spanned::new(string_id, r.span);
                let type_args: Option<&'static [Type<'static>]> =
                    r.type_arguments.as_ref().map(|args| {
                        let vec: Vec<Type<'static>> =
                            args.iter().map(|t| t.to_type(interner)).collect();
                        &*Box::leak(vec.into_boxed_slice())
                    });
                TypeKind::Reference(TypeReference {
                    name: ident,
                    type_arguments: type_args,
                    span: r.span,
                })
            }
            SerializableTypeKind::Union(members) => {
                let vec: Vec<Type<'static>> = members.iter().map(|t| t.to_type(interner)).collect();
                TypeKind::Union(&*Box::leak(vec.into_boxed_slice()))
            }
            SerializableTypeKind::Intersection(members) => {
                let vec: Vec<Type<'static>> = members.iter().map(|t| t.to_type(interner)).collect();
                TypeKind::Intersection(&*Box::leak(vec.into_boxed_slice()))
            }
            SerializableTypeKind::Object(obj) => TypeKind::Object(obj.to_object_type(interner)),
            SerializableTypeKind::Array(elem) => {
                let inner: &'static Type<'static> = Box::leak(Box::new(elem.to_type(interner)));
                TypeKind::Array(inner)
            }
            SerializableTypeKind::Tuple(elems) => {
                let vec: Vec<Type<'static>> = elems.iter().map(|t| t.to_type(interner)).collect();
                TypeKind::Tuple(&*Box::leak(vec.into_boxed_slice()))
            }
            SerializableTypeKind::Function(func) => {
                TypeKind::Function(func.to_function_type(interner))
            }
            SerializableTypeKind::Literal(lit) => TypeKind::Literal(lit.to_literal()),
            SerializableTypeKind::Nullable(inner) => {
                let inner_ref: &'static Type<'static> =
                    Box::leak(Box::new(inner.to_type(interner)));
                TypeKind::Nullable(inner_ref)
            }
            SerializableTypeKind::Namespace(parts) => TypeKind::Namespace(parts.clone()),
            SerializableTypeKind::Unknown => TypeKind::Primitive(PrimitiveType::Unknown),
        }
    }
}

// ---------------------------------------------------------------------------
// Supporting type conversions
// ---------------------------------------------------------------------------

impl SerializableObjectType {
    fn from_object(obj: &ObjectType<'_>, interner: &StringInterner) -> Self {
        SerializableObjectType {
            members: obj
                .members
                .iter()
                .map(|m| SerializableObjectMember::from_member(m, interner))
                .collect(),
            span: obj.span,
        }
    }

    fn to_object_type(&self, interner: &StringInterner) -> ObjectType<'static> {
        let members: Vec<ObjectTypeMember<'static>> =
            self.members.iter().map(|m| m.to_member(interner)).collect();
        ObjectType {
            members: &*Box::leak(members.into_boxed_slice()),
            span: self.span,
        }
    }
}

impl SerializableObjectMember {
    fn from_member(member: &ObjectTypeMember<'_>, interner: &StringInterner) -> Self {
        match member {
            ObjectTypeMember::Property(prop) => {
                SerializableObjectMember::Property(SerializableProperty {
                    is_readonly: prop.is_readonly,
                    name: interner.resolve(prop.name.node),
                    is_optional: prop.is_optional,
                    type_annotation: SerializableType::from_type(&prop.type_annotation, interner),
                    span: prop.span,
                })
            }
            ObjectTypeMember::Method(method) => {
                SerializableObjectMember::Method(SerializableMethod {
                    name: interner.resolve(method.name.node),
                    parameters: method
                        .parameters
                        .iter()
                        .map(|p| SerializableParameter::from_param(p, interner))
                        .collect(),
                    return_type: SerializableType::from_type(&method.return_type, interner),
                    span: method.span,
                })
            }
            ObjectTypeMember::Index(idx) => SerializableObjectMember::Index(SerializableIndex {
                key_name: interner.resolve(idx.key_name.node),
                key_type: idx.key_type,
                value_type: SerializableType::from_type(&idx.value_type, interner),
                span: idx.span,
            }),
        }
    }

    fn to_member(&self, interner: &StringInterner) -> ObjectTypeMember<'static> {
        match self {
            SerializableObjectMember::Property(prop) => {
                let name_id = interner.intern(&prop.name);
                let ident = Spanned::new(name_id, prop.span);
                ObjectTypeMember::Property(luanext_parser::ast::statement::PropertySignature {
                    is_readonly: prop.is_readonly,
                    name: ident,
                    is_optional: prop.is_optional,
                    type_annotation: prop.type_annotation.to_type(interner),
                    span: prop.span,
                })
            }
            SerializableObjectMember::Method(method) => {
                let name_id = interner.intern(&method.name);
                let ident = Spanned::new(name_id, method.span);
                let params: Vec<luanext_parser::ast::statement::Parameter<'static>> = method
                    .parameters
                    .iter()
                    .map(|p| p.to_parameter(interner))
                    .collect();
                ObjectTypeMember::Method(luanext_parser::ast::statement::MethodSignature {
                    name: ident,
                    type_parameters: None,
                    parameters: &*Box::leak(params.into_boxed_slice()),
                    return_type: method.return_type.to_type(interner),
                    body: None,
                    span: method.span,
                })
            }
            SerializableObjectMember::Index(idx) => {
                let key_id = interner.intern(&idx.key_name);
                let key_ident = Spanned::new(key_id, idx.span);
                ObjectTypeMember::Index(luanext_parser::ast::statement::IndexSignature {
                    key_name: key_ident,
                    key_type: idx.key_type,
                    value_type: idx.value_type.to_type(interner),
                    span: idx.span,
                })
            }
        }
    }
}

impl SerializableFunctionType {
    fn from_function(func: &FunctionType<'_>, interner: &StringInterner) -> Self {
        SerializableFunctionType {
            type_parameters: func.type_parameters.map(|tps| {
                tps.iter()
                    .map(|tp| SerializableTypeParameter {
                        name: interner.resolve(tp.name.node),
                        constraint: tp
                            .constraint
                            .map(|c| Box::new(SerializableType::from_type(c, interner))),
                        default: tp
                            .default
                            .map(|d| Box::new(SerializableType::from_type(d, interner))),
                        span: tp.span,
                    })
                    .collect()
            }),
            parameters: func
                .parameters
                .iter()
                .map(|p| SerializableParameter::from_param(p, interner))
                .collect(),
            return_type: Box::new(SerializableType::from_type(func.return_type, interner)),
            throws: func.throws.map(|throws| {
                throws
                    .iter()
                    .map(|t| SerializableType::from_type(t, interner))
                    .collect()
            }),
            span: func.span,
        }
    }

    fn to_function_type(&self, interner: &StringInterner) -> FunctionType<'static> {
        let type_params: Option<&'static [luanext_parser::ast::statement::TypeParameter<'static>]> =
            self.type_parameters.as_ref().map(|tps| {
                let vec: Vec<luanext_parser::ast::statement::TypeParameter<'static>> = tps
                    .iter()
                    .map(|tp| {
                        let name_id = interner.intern(&tp.name);
                        let ident = Spanned::new(name_id, tp.span);
                        let constraint: Option<&'static Type<'static>> = tp
                            .constraint
                            .as_ref()
                            .map(|c| &*Box::leak(Box::new(c.to_type(interner))));
                        let default: Option<&'static Type<'static>> = tp
                            .default
                            .as_ref()
                            .map(|d| &*Box::leak(Box::new(d.to_type(interner))));
                        luanext_parser::ast::statement::TypeParameter {
                            name: ident,
                            constraint,
                            default,
                            span: tp.span,
                        }
                    })
                    .collect();
                &*Box::leak(vec.into_boxed_slice())
            });

        let params: Vec<luanext_parser::ast::statement::Parameter<'static>> = self
            .parameters
            .iter()
            .map(|p| p.to_parameter(interner))
            .collect();

        let return_type: &'static Type<'static> =
            Box::leak(Box::new(self.return_type.to_type(interner)));

        let throws: Option<&'static [Type<'static>]> = self.throws.as_ref().map(|ts| {
            let vec: Vec<Type<'static>> = ts.iter().map(|t| t.to_type(interner)).collect();
            &*Box::leak(vec.into_boxed_slice())
        });

        FunctionType {
            type_parameters: type_params,
            parameters: &*Box::leak(params.into_boxed_slice()),
            return_type,
            throws,
            span: self.span,
        }
    }
}

impl SerializableParameter {
    fn from_param(
        param: &luanext_parser::ast::statement::Parameter<'_>,
        interner: &StringInterner,
    ) -> Self {
        // Extract parameter name from pattern
        let name = match &param.pattern {
            luanext_parser::ast::pattern::Pattern::Identifier(ident) => {
                interner.resolve(ident.node)
            }
            _ => "_".to_string(),
        };
        SerializableParameter {
            name,
            type_annotation: param
                .type_annotation
                .as_ref()
                .map(|t| SerializableType::from_type(t, interner)),
            is_rest: param.is_rest,
            is_optional: param.is_optional,
            span: param.span,
        }
    }

    fn to_parameter(
        &self,
        interner: &StringInterner,
    ) -> luanext_parser::ast::statement::Parameter<'static> {
        let name_id = interner.intern(&self.name);
        let ident: Ident = Spanned::new(name_id, self.span);
        luanext_parser::ast::statement::Parameter {
            pattern: luanext_parser::ast::pattern::Pattern::Identifier(ident),
            type_annotation: self.type_annotation.as_ref().map(|t| t.to_type(interner)),
            default: None,
            is_rest: self.is_rest,
            is_optional: self.is_optional,
            span: self.span,
        }
    }
}

impl SerializableLiteral {
    fn from_literal(lit: &Literal) -> Self {
        match lit {
            Literal::Nil => SerializableLiteral::Nil,
            Literal::Boolean(b) => SerializableLiteral::Boolean(*b),
            Literal::Number(n) => SerializableLiteral::Number(*n),
            Literal::Integer(i) => SerializableLiteral::Integer(*i),
            Literal::String(s) => SerializableLiteral::String(s.clone()),
        }
    }

    fn to_literal(&self) -> Literal {
        match self {
            SerializableLiteral::Nil => Literal::Nil,
            SerializableLiteral::Boolean(b) => Literal::Boolean(*b),
            SerializableLiteral::Number(n) => Literal::Number(*n),
            SerializableLiteral::Integer(i) => Literal::Integer(*i),
            SerializableLiteral::String(s) => Literal::String(s.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// ModuleExports conversion
// ---------------------------------------------------------------------------

impl SerializableExportedSymbol {
    fn to_exported(&self, interner: &StringInterner) -> (String, ExportedSymbol) {
        let typ = self.typ.to_type(interner);
        let symbol = Symbol::new(self.name.clone(), self.kind, typ, self.span);
        let mut sym = symbol;
        sym.is_exported = self.is_exported;
        (
            self.name.clone(),
            ExportedSymbol::new(sym, self.is_type_only),
        )
    }
}

impl SerializableModuleExports {
    /// Convert `ModuleExports` to a serializable form.
    ///
    /// Should be called during type checking when the interner is available.
    pub fn from_exports(exports: &ModuleExports, interner: &StringInterner) -> Self {
        let named = exports
            .named
            .iter()
            .map(|(name, export)| {
                let sym = SerializableExportedSymbol {
                    name: name.clone(),
                    kind: export.symbol.kind,
                    typ: SerializableType::from_type(&export.symbol.typ, interner),
                    span: export.symbol.span,
                    is_exported: export.symbol.is_exported,
                    is_type_only: export.is_type_only,
                };
                (name.clone(), sym)
            })
            .collect();

        let default = exports
            .default
            .as_ref()
            .map(|export| SerializableExportedSymbol {
                name: "default".to_string(),
                kind: export.symbol.kind,
                typ: SerializableType::from_type(&export.symbol.typ, interner),
                span: export.symbol.span,
                is_exported: export.symbol.is_exported,
                is_type_only: export.is_type_only,
            });

        SerializableModuleExports { named, default }
    }

    /// Reconstruct `ModuleExports` from the serialized form.
    pub fn to_exports(&self, interner: &StringInterner) -> ModuleExports {
        let mut exports = ModuleExports::new();

        for (name, ser_sym) in &self.named {
            let (_, exported) = ser_sym.to_exported(interner);
            exports.add_named(name.clone(), exported);
        }

        if let Some(ref ser_default) = self.default {
            let (_, exported) = ser_default.to_exported(interner);
            exports.set_default(exported);
        }

        exports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interner() -> StringInterner {
        StringInterner::new()
    }

    #[test]
    fn test_primitive_roundtrip() {
        let interner = make_interner();
        let ty = Type::new(TypeKind::Primitive(PrimitiveType::Number), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        assert!(matches!(
            restored.kind,
            TypeKind::Primitive(PrimitiveType::Number)
        ));
    }

    #[test]
    fn test_array_roundtrip() {
        let interner = make_interner();
        // Construct Array(Number) using Box::leak for the test
        let inner: &'static Type<'static> = Box::leak(Box::new(Type::new(
            TypeKind::Primitive(PrimitiveType::Number),
            Span::default(),
        )));
        let ty = Type::new(TypeKind::Array(inner), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Array(elem) => {
                assert!(matches!(
                    elem.kind,
                    TypeKind::Primitive(PrimitiveType::Number)
                ));
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_union_roundtrip() {
        let interner = make_interner();
        let members: &'static [Type<'static>] = Box::leak(
            vec![
                Type::new(TypeKind::Primitive(PrimitiveType::Number), Span::default()),
                Type::new(TypeKind::Primitive(PrimitiveType::String), Span::default()),
            ]
            .into_boxed_slice(),
        );
        let ty = Type::new(TypeKind::Union(members), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Union(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(
                    elems[0].kind,
                    TypeKind::Primitive(PrimitiveType::Number)
                ));
                assert!(matches!(
                    elems[1].kind,
                    TypeKind::Primitive(PrimitiveType::String)
                ));
            }
            _ => panic!("Expected Union type"),
        }
    }

    #[test]
    fn test_reference_roundtrip() {
        let interner = make_interner();
        let name_id = interner.intern("MyType");
        let ident = Spanned::new(name_id, Span::default());
        let ty = Type::new(
            TypeKind::Reference(TypeReference {
                name: ident,
                type_arguments: None,
                span: Span::default(),
            }),
            Span::default(),
        );

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Reference(r) => {
                let name = interner.resolve(r.name.node);
                assert_eq!(name, "MyType");
            }
            _ => panic!("Expected Reference type"),
        }
    }

    #[test]
    fn test_function_roundtrip() {
        let interner = make_interner();
        let return_type: &'static Type<'static> = Box::leak(Box::new(Type::new(
            TypeKind::Primitive(PrimitiveType::Void),
            Span::default(),
        )));
        let params: &'static [luanext_parser::ast::statement::Parameter<'static>] =
            &*Box::leak(Vec::new().into_boxed_slice());

        let func_type = FunctionType {
            type_parameters: None,
            parameters: params,
            return_type,
            throws: None,
            span: Span::default(),
        };
        let ty = Type::new(TypeKind::Function(func_type), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Function(f) => {
                assert!(matches!(
                    f.return_type.kind,
                    TypeKind::Primitive(PrimitiveType::Void)
                ));
                assert_eq!(f.parameters.len(), 0);
            }
            _ => panic!("Expected Function type"),
        }
    }

    #[test]
    fn test_unknown_fallback_for_complex_types() {
        let interner = make_interner();
        // KeyOf falls back to Unknown
        let inner: &'static Type<'static> = Box::leak(Box::new(Type::new(
            TypeKind::Primitive(PrimitiveType::String),
            Span::default(),
        )));
        let ty = Type::new(TypeKind::KeyOf(inner), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        assert!(matches!(ser.kind, SerializableTypeKind::Unknown));

        let restored = ser.to_type(&interner);
        assert!(matches!(
            restored.kind,
            TypeKind::Primitive(PrimitiveType::Unknown)
        ));
    }

    #[test]
    fn test_serializable_module_exports_roundtrip() {
        let interner = make_interner();
        let mut exports = ModuleExports::new();

        let typ = Type::new(TypeKind::Primitive(PrimitiveType::Number), Span::default());
        let symbol = Symbol::new(
            "foo".to_string(),
            SymbolKind::Variable,
            typ,
            Span::default(),
        );
        exports.add_named("foo".to_string(), ExportedSymbol::new(symbol, false));

        let ser = SerializableModuleExports::from_exports(&exports, &interner);
        let bytes = bincode::serialize(&ser).expect("serialization should work");
        let deser: SerializableModuleExports =
            bincode::deserialize(&bytes).expect("deserialization should work");

        let restored = deser.to_exports(&interner);
        assert!(restored.get_named("foo").is_some());
        assert_eq!(restored.get_named("foo").unwrap().symbol.name, "foo");
        assert_eq!(
            restored.get_named("foo").unwrap().symbol.kind,
            SymbolKind::Variable
        );
    }

    #[test]
    fn test_literal_roundtrip() {
        let interner = make_interner();

        let ty = Type::new(
            TypeKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected Literal String type"),
        }
    }

    #[test]
    fn test_nullable_roundtrip() {
        let interner = make_interner();
        let inner: &'static Type<'static> = Box::leak(Box::new(Type::new(
            TypeKind::Primitive(PrimitiveType::String),
            Span::default(),
        )));
        let ty = Type::new(TypeKind::Nullable(inner), Span::default());

        let ser = SerializableType::from_type(&ty, &interner);
        let restored = ser.to_type(&interner);

        match &restored.kind {
            TypeKind::Nullable(inner) => {
                assert!(matches!(
                    inner.kind,
                    TypeKind::Primitive(PrimitiveType::String)
                ));
            }
            _ => panic!("Expected Nullable type"),
        }
    }
}
