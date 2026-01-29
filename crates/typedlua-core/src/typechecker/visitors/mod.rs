mod access_control;
mod inference;

pub use super::generics::{GenericInstantiator, GenericVisitor};
pub use super::narrowing::{NarrowingContext, NarrowingVisitor, TypeNarrower};
pub use access_control::{
    AccessControl, AccessControlVisitor, ClassContext, ClassMemberInfo, ClassMemberKind,
};
pub use inference::{TypeInferenceVisitor, TypeInferrer};

pub trait TypeCheckVisitor {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}
