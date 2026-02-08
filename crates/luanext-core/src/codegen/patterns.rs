use crate::codegen::CodeGenerator;
use luanext_parser::ast::pattern::{
    ArrayPattern, ArrayPatternElement, ObjectPattern, Pattern, PatternWithDefault,
};

impl CodeGenerator {
    pub fn generate_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => {
                let name_str = self.resolve(name.node);
                self.write(&name_str);
            }
            Pattern::Wildcard(_) => {
                self.write("_");
            }
            Pattern::Literal(lit, _) => {
                self.generate_literal(lit);
            }
            Pattern::Array(array_pattern) => {
                self.generate_array_pattern(array_pattern);
            }
            Pattern::Object(object_pattern) => {
                self.generate_object_pattern(object_pattern);
            }
            Pattern::Or(or_pattern) => {
                // OR patterns are only valid in match expressions, not in variable
                // declarations or destructuring assignments.
                // If we encounter one here, it's an error that should have been caught
                // during type checking.
                eprintln!(
                    "Error at {:?}: OR patterns (|) are only valid in match expressions, \
                    not in variable declarations or destructuring assignments",
                    or_pattern.span
                );
                // Generate wildcard as fallback for error recovery
                self.write("_");
            }
        }
    }

    pub fn generate_array_pattern(&mut self, pattern: &ArrayPattern) {
        for (i, elem) in pattern.elements.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            match elem {
                ArrayPatternElement::Pattern(PatternWithDefault { pattern: pat, .. }) => {
                    self.generate_pattern(pat);
                }
                ArrayPatternElement::Rest(_) => {
                    self.write("...");
                }
                ArrayPatternElement::Hole => {
                    self.write("_");
                }
            }
        }
    }

    pub fn generate_object_pattern(&mut self, pattern: &ObjectPattern) {
        for (i, prop) in pattern.properties.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            let key_str = self.resolve(prop.key.node);

            if let Some(value_pattern) = &prop.value {
                if matches!(value_pattern, Pattern::Identifier(id) if id.node == prop.key.node) {
                    self.write(&key_str);
                } else {
                    self.write(&key_str);
                    self.write(" = ");
                    self.generate_pattern(value_pattern);
                }
            } else {
                self.write(&key_str);
            }

            if let Some(default_expr) = &prop.default {
                self.write(" = ");
                self.generate_expression(default_expr);
            }
        }
    }
}
