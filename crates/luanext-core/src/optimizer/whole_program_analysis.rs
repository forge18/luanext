//! Whole-program analysis infrastructure for parallel optimization
//!
//! This module provides analysis results that require cross-module information.
//! Analysis is built once sequentially, then shared (read-only) across parallel
//! optimization passes via Arc.

use crate::config::OptimizationLevel;
use crate::optimizer::devirtualization::ClassHierarchy;
use luanext_parser::ast::Program;
use std::sync::Arc;

/// Thread-safe whole-program analysis results
///
/// This struct contains analysis that requires cross-module information.
/// It's built once sequentially before parallel code generation, then shared
/// (read-only) across parallel optimization passes.
#[derive(Clone, Debug)]
pub struct WholeProgramAnalysis {
    /// Class hierarchy for devirtualization
    pub class_hierarchy: Arc<ClassHierarchy>,
    /// Cross-module side-effect information (populated when analysis infrastructure is enabled)
    pub side_effects: Option<Arc<super::analysis::SideEffectInfo>>,
}

impl WholeProgramAnalysis {
    /// Build whole-program analysis by scanning all type-checked modules
    ///
    /// This should be called sequentially after type checking, before parallel
    /// code generation begins. The resulting analysis is thread-safe and can
    /// be cloned cheaply (Arc) for each parallel worker.
    pub fn build<'arena>(
        programs: &[&Program<'arena>],
        optimization_level: OptimizationLevel,
    ) -> Self {
        // Only build expensive analysis if O3+ optimization is enabled
        let class_hierarchy = if optimization_level >= OptimizationLevel::Aggressive {
            ClassHierarchy::build_multi_module(programs)
        } else {
            ClassHierarchy::default()
        };

        Self {
            class_hierarchy: Arc::new(class_hierarchy),
            side_effects: None,
        }
    }
}
