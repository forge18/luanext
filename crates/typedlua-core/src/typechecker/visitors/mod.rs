mod access_control;

pub use access_control::{
    AccessControl, AccessControlVisitor, ClassContext, ClassMemberInfo, ClassMemberKind,
};

pub trait TypeCheckVisitor {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}
