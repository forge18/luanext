use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::serializable_types::SerializableModuleExports;
use super::{CacheError, Result};

/// Cached module data
///
/// Stores enough information to reconstruct module exports for the
/// `ModuleRegistry` on cache hits, avoiding re-parsing and re-type-checking
/// unchanged files. The `serializable_exports` field contains the full
/// typed export data; older cache entries may have `None` and will
/// fall through to recompilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModule {
    /// Module identifier (canonical path)
    pub path: PathBuf,

    /// Hash of the source file for cache invalidation
    pub source_hash: String,

    /// Interned string table — needed to reconstruct a StringInterner
    /// so that StringId values resolve correctly.
    pub interner_strings: Vec<String>,

    /// Serialized export names (simplified representation)
    pub export_names: Vec<String>,

    /// Whether a default export exists
    pub has_default_export: bool,

    /// Full serializable export data for cache-hit module registry population.
    /// `None` for caches created before this field was added — those entries
    /// fall through to recompilation.
    #[serde(default)]
    pub serializable_exports: Option<SerializableModuleExports>,
}

impl CachedModule {
    /// Create a new cached module
    pub fn new(
        path: PathBuf,
        source_hash: String,
        interner_strings: Vec<String>,
        export_names: Vec<String>,
        has_default_export: bool,
        serializable_exports: Option<SerializableModuleExports>,
    ) -> Self {
        Self {
            path,
            source_hash,
            interner_strings,
            export_names,
            has_default_export,
            serializable_exports,
        }
    }

    /// Serialize to binary format
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(CacheError::from)
    }

    /// Deserialize from binary format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(CacheError::from)
    }

    /// Compute hash of cached module data (for integrity checking)
    pub fn compute_hash(&self) -> String {
        let bytes = self.to_bytes().unwrap_or_default();
        let hash = blake3::hash(&bytes);
        hash.to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_module() -> CachedModule {
        CachedModule::new(
            PathBuf::from("/test/module.luax"),
            "abc123".to_string(),
            vec![],
            vec![],
            false,
            None,
        )
    }

    #[test]
    fn test_cached_module_serialization() {
        let module = make_test_module();

        let bytes = module.to_bytes().unwrap();
        let deserialized = CachedModule::from_bytes(&bytes).unwrap();

        assert_eq!(module.path, deserialized.path);
        assert_eq!(module.source_hash, deserialized.source_hash);
    }

    #[test]
    fn test_compute_hash_consistency() {
        let module = make_test_module();

        let hash1 = module.compute_hash();
        let hash2 = module.compute_hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_serialization_with_exports() {
        let exports = SerializableModuleExports {
            named: vec![],
            default: None,
        };
        let module = CachedModule::new(
            PathBuf::from("/test/module.luax"),
            "abc123".to_string(),
            vec!["foo".to_string()],
            vec!["foo".to_string()],
            false,
            Some(exports),
        );

        let bytes = module.to_bytes().unwrap();
        let deserialized = CachedModule::from_bytes(&bytes).unwrap();

        assert!(deserialized.serializable_exports.is_some());
    }

    #[test]
    fn test_backward_compat_without_exports() {
        // Simulate old cache format without serializable_exports
        let module_old = CachedModule {
            path: PathBuf::from("/test/module.luax"),
            source_hash: "abc123".to_string(),
            interner_strings: vec![],
            export_names: vec![],
            has_default_export: false,
            serializable_exports: None,
        };

        let bytes = module_old.to_bytes().unwrap();
        let deserialized = CachedModule::from_bytes(&bytes).unwrap();

        assert!(deserialized.serializable_exports.is_none());
    }
}
