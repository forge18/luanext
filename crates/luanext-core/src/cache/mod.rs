//! Incremental compilation cache for LuaNext
//!
//! This module provides functionality to cache type-checked modules to disk,
//! enabling faster incremental compilation by only recompiling changed files
//! and their dependents.

mod error;
mod hash;
mod invalidation;
mod manager;
mod manifest;
mod module;
pub mod serializable_types;

pub use error::{CacheError, Result};
pub use hash::{hash_config, hash_file};
pub use invalidation::InvalidationEngine;
pub use manager::CacheManager;
pub use manifest::{CacheEntry, CacheManifest};
pub use module::CachedModule;
pub use serializable_types::SerializableModuleExports;

/// Cache format version - increment when cache structure changes
/// v2: Added serializable_exports field to CachedModule
pub const CACHE_VERSION: u32 = 2;

/// Default cache directory name
pub const CACHE_DIR_NAME: &str = ".luanext-cache";

/// Cache manifest file name
pub const MANIFEST_FILE_NAME: &str = "manifest.bin";

/// Cache modules subdirectory name
pub const MODULES_DIR_NAME: &str = "modules";
