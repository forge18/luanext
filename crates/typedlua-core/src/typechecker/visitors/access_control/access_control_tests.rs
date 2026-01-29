#[cfg(test)]
mod tests {
    use crate::typechecker::visitors::{
        AccessControl, AccessControlVisitor, ClassContext, ClassMemberInfo, ClassMemberKind,
        TypeCheckVisitor,
    };
    use typedlua_parser::ast::statement::AccessModifier;
    use typedlua_parser::ast::types::{PrimitiveType, Type, TypeKind};
    use typedlua_parser::span::Span;

    fn create_test_member(name: &str, access: AccessModifier) -> ClassMemberInfo {
        ClassMemberInfo {
            name: name.to_string(),
            access,
            _is_static: false,
            kind: ClassMemberKind::Property {
                type_annotation: Type::new(
                    TypeKind::Primitive(PrimitiveType::Number),
                    Span::default(),
                ),
            },
            is_final: false,
        }
    }

    fn create_test_method(name: &str, access: AccessModifier) -> ClassMemberInfo {
        ClassMemberInfo {
            name: name.to_string(),
            access,
            _is_static: false,
            kind: ClassMemberKind::Method {
                parameters: vec![],
                return_type: None,
            },
            is_final: false,
        }
    }

    #[test]
    fn test_access_control_visitor_name() {
        let access_control = AccessControl::new();
        assert_eq!(access_control.name(), "AccessControl");
    }

    #[test]
    fn test_public_member_accessible_from_anywhere() {
        let mut access_control = AccessControl::new();

        // Register a class with a public member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("publicProp", AccessModifier::Public),
        );

        // Access from outside the class (no current class context)
        let current_class: Option<ClassContext> = None;
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "publicProp",
            Span::default(),
        );

        assert!(
            result.is_ok(),
            "Public members should be accessible from anywhere"
        );
    }

    #[test]
    fn test_private_member_accessible_within_same_class() {
        let mut access_control = AccessControl::new();

        // Register a class with a private member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("privateProp", AccessModifier::Private),
        );

        // Access from within the same class
        let current_class = Some(ClassContext {
            name: "MyClass".to_string(),
            parent: None,
        });
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "privateProp",
            Span::default(),
        );

        assert!(
            result.is_ok(),
            "Private members should be accessible within the same class"
        );
    }

    #[test]
    fn test_private_member_not_accessible_from_other_class() {
        let mut access_control = AccessControl::new();

        // Register a class with a private member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("privateProp", AccessModifier::Private),
        );

        // Try to access from a different class
        let current_class = Some(ClassContext {
            name: "OtherClass".to_string(),
            parent: None,
        });
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "privateProp",
            Span::default(),
        );

        assert!(
            result.is_err(),
            "Private members should not be accessible from other classes"
        );
        let err = result.unwrap_err();
        assert!(
            err.message.contains("private"),
            "Error message should mention 'private'"
        );
        assert!(
            err.message.contains("privateProp"),
            "Error message should mention the member name"
        );
    }

    #[test]
    fn test_private_member_not_accessible_from_outside() {
        let mut access_control = AccessControl::new();

        // Register a class with a private member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("privateProp", AccessModifier::Private),
        );

        // Try to access from outside any class
        let current_class: Option<ClassContext> = None;
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "privateProp",
            Span::default(),
        );

        assert!(
            result.is_err(),
            "Private members should not be accessible from outside classes"
        );
    }

    #[test]
    fn test_protected_member_accessible_within_same_class() {
        let mut access_control = AccessControl::new();

        // Register a class with a protected member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("protectedProp", AccessModifier::Protected),
        );

        // Access from within the same class
        let current_class = Some(ClassContext {
            name: "MyClass".to_string(),
            parent: None,
        });
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "protectedProp",
            Span::default(),
        );

        assert!(
            result.is_ok(),
            "Protected members should be accessible within the same class"
        );
    }

    #[test]
    fn test_protected_member_accessible_from_subclass() {
        let mut access_control = AccessControl::new();

        // Register a parent class with a protected member
        access_control.register_class("ParentClass", None);
        access_control.register_member(
            "ParentClass",
            create_test_member("protectedProp", AccessModifier::Protected),
        );

        // Register a child class
        access_control.register_class("ChildClass", Some("ParentClass".to_string()));

        // Access from the child class context
        let current_class = Some(ClassContext {
            name: "ChildClass".to_string(),
            parent: Some("ParentClass".to_string()),
        });

        // Set the current class for is_subclass check
        access_control.set_current_class(current_class.clone());

        let result = access_control.check_member_access(
            &current_class,
            "ParentClass",
            "protectedProp",
            Span::default(),
        );

        assert!(
            result.is_ok(),
            "Protected members should be accessible from subclasses"
        );
    }

    #[test]
    fn test_protected_member_not_accessible_from_unrelated_class() {
        let mut access_control = AccessControl::new();

        // Register a class with a protected member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("protectedProp", AccessModifier::Protected),
        );

        // Try to access from an unrelated class
        let current_class = Some(ClassContext {
            name: "OtherClass".to_string(),
            parent: None,
        });
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "protectedProp",
            Span::default(),
        );

        assert!(
            result.is_err(),
            "Protected members should not be accessible from unrelated classes"
        );
        let err = result.unwrap_err();
        assert!(
            err.message.contains("protected"),
            "Error message should mention 'protected'"
        );
    }

    #[test]
    fn test_protected_member_not_accessible_from_outside() {
        let mut access_control = AccessControl::new();

        // Register a class with a protected member
        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("protectedProp", AccessModifier::Protected),
        );

        // Try to access from outside any class
        let current_class: Option<ClassContext> = None;
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "protectedProp",
            Span::default(),
        );

        assert!(
            result.is_err(),
            "Protected members should not be accessible from outside classes"
        );
    }

    #[test]
    fn test_member_not_found_allows_access() {
        let access_control = AccessControl::new();

        // Try to access a member that doesn't exist
        let current_class: Option<ClassContext> = None;
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "nonExistentProp",
            Span::default(),
        );

        assert!(result.is_ok(), "Access to unknown members should be allowed (for interface/unknown class compatibility)");
    }

    #[test]
    fn test_class_registration() {
        let mut access_control = AccessControl::new();

        access_control.register_class("TestClass", Some("ParentClass".to_string()));

        // Verify class is registered by checking we can add members
        access_control.register_member(
            "TestClass",
            create_test_member("prop", AccessModifier::Public),
        );
        let members = access_control.get_class_members("TestClass");
        assert!(members.is_some(), "Class should be registered");
        assert_eq!(members.unwrap().len(), 1, "Class should have one member");
    }

    #[test]
    fn test_final_class_marking() {
        let mut access_control = AccessControl::new();

        access_control.register_class("FinalClass", None);
        assert!(
            !access_control.is_class_final("FinalClass"),
            "Class should not be final by default"
        );

        access_control.mark_class_final("FinalClass", true);
        assert!(
            access_control.is_class_final("FinalClass"),
            "Class should be marked as final"
        );

        access_control.mark_class_final("FinalClass", false);
        assert!(
            !access_control.is_class_final("FinalClass"),
            "Class should no longer be final"
        );
    }

    #[test]
    fn test_current_class_context() {
        let mut access_control = AccessControl::new();

        // Initially no current class
        assert!(
            access_control.get_current_class().is_none(),
            "Should have no current class initially"
        );

        // Set current class
        let context = Some(ClassContext {
            name: "MyClass".to_string(),
            parent: Some("ParentClass".to_string()),
        });
        access_control.set_current_class(context.clone());

        let retrieved = access_control.get_current_class();
        assert!(retrieved.is_some(), "Should have a current class");
        assert_eq!(
            retrieved.as_ref().unwrap().name,
            "MyClass",
            "Current class name should match"
        );
        assert_eq!(
            retrieved.as_ref().unwrap().parent,
            Some("ParentClass".to_string()),
            "Parent class should match"
        );

        // Clear current class
        access_control.set_current_class(None);
        assert!(
            access_control.get_current_class().is_none(),
            "Current class should be cleared"
        );
    }

    #[test]
    fn test_is_subclass_direct_parent() {
        let mut access_control = AccessControl::new();

        // Register parent and child
        access_control.register_class("ParentClass", None);
        access_control.register_class("ChildClass", Some("ParentClass".to_string()));

        // Set current class context for the child
        access_control.set_current_class(Some(ClassContext {
            name: "ChildClass".to_string(),
            parent: Some("ParentClass".to_string()),
        }));

        assert!(
            access_control.is_subclass("ChildClass", "ParentClass"),
            "ChildClass should be a subclass of ParentClass"
        );
        assert!(
            !access_control.is_subclass("ParentClass", "ChildClass"),
            "ParentClass should not be a subclass of ChildClass"
        );
    }

    #[test]
    fn test_is_subclass_unrelated_classes() {
        let mut access_control = AccessControl::new();

        access_control.register_class("ClassA", None);
        access_control.register_class("ClassB", None);

        access_control.set_current_class(Some(ClassContext {
            name: "ClassA".to_string(),
            parent: None,
        }));

        assert!(
            !access_control.is_subclass("ClassA", "ClassB"),
            "Unrelated classes should not be subclasses"
        );
    }

    #[test]
    fn test_multiple_members_same_class() {
        let mut access_control = AccessControl::new();

        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("publicProp", AccessModifier::Public),
        );
        access_control.register_member(
            "MyClass",
            create_test_member("privateProp", AccessModifier::Private),
        );
        access_control.register_member(
            "MyClass",
            create_test_member("protectedProp", AccessModifier::Protected),
        );

        let members = access_control.get_class_members("MyClass").unwrap();
        assert_eq!(members.len(), 3, "Class should have three members");
    }

    #[test]
    fn test_method_member_access() {
        let mut access_control = AccessControl::new();

        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_method("publicMethod", AccessModifier::Public),
        );
        access_control.register_member(
            "MyClass",
            create_test_method("privateMethod", AccessModifier::Private),
        );

        let current_class = Some(ClassContext {
            name: "MyClass".to_string(),
            parent: None,
        });

        // Public method should be accessible
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "publicMethod",
            Span::default(),
        );
        assert!(result.is_ok(), "Public method should be accessible");

        // Private method should be accessible within same class
        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "privateMethod",
            Span::default(),
        );
        assert!(
            result.is_ok(),
            "Private method should be accessible within same class"
        );
    }

    #[test]
    fn test_error_message_contains_relevant_info() {
        let mut access_control = AccessControl::new();

        access_control.register_class("MyClass", None);
        access_control.register_member(
            "MyClass",
            create_test_member("secret", AccessModifier::Private),
        );

        let current_class = Some(ClassContext {
            name: "OtherClass".to_string(),
            parent: None,
        });

        let result = access_control.check_member_access(
            &current_class,
            "MyClass",
            "secret",
            Span::default(),
        );
        let err = result.unwrap_err();

        assert!(
            err.message.contains("secret"),
            "Error should mention the member name"
        );
        assert!(
            err.message.contains("MyClass"),
            "Error should mention the class name"
        );
        assert!(
            err.message.contains("private"),
            "Error should mention the access modifier"
        );
    }
}
