use crate::config::OptimizationLevel;
use crate::diagnostics::DiagnosticHandler;

use std::rc::Rc;
use std::sync::Arc;
use typedlua_parser::ast::Program;
use typedlua_parser::string_interner::StringInterner;

mod constant_folding;
pub use constant_folding::ConstantFoldingPass;

mod dead_code_elimination;
pub use dead_code_elimination::DeadCodeEliminationPass;

mod algebraic_simplification;
pub use algebraic_simplification::AlgebraicSimplificationPass;

mod table_preallocation;
pub use table_preallocation::TablePreallocationPass;

mod global_localization;
pub use global_localization::GlobalLocalizationPass;

mod function_inlining;
pub use function_inlining::FunctionInliningPass;

mod loop_optimization;
pub use loop_optimization::LoopOptimizationPass;

mod string_concat_optimization;
pub use string_concat_optimization::StringConcatOptimizationPass;

mod dead_store_elimination;
pub use dead_store_elimination::DeadStoreEliminationPass;

mod tail_call_optimization;
pub use tail_call_optimization::TailCallOptimizationPass;

mod generic_specialization;
pub use generic_specialization::GenericSpecializationPass;

mod rich_enum_optimization;
pub use rich_enum_optimization::*;

mod method_to_function_conversion;
pub use method_to_function_conversion::*;

mod devirtualization;
pub use devirtualization::DevirtualizationPass;

mod operator_inlining;
pub use operator_inlining::OperatorInliningPass;

mod interface_inlining;
pub use interface_inlining::InterfaceMethodInliningPass;

mod aggressive_inlining;
pub use aggressive_inlining::AggressiveInliningPass;