use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{CacheError, Result, CACHE_VERSION};

/// Cache manifest containing metadata and dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Schema version for cache format
    pub version: u32,

    /// Hash of compiler configuration (invalidate on config change)
    pub config_hash: String,

    /// Cached modules: canonical path -> cache entry
    pub modules: FxHashMap<PathBuf, CacheEntry>,

    /// Dependency graph: module path -> list of dependency paths
    pub dependencies: FxHashMap<PathBuf, Vec<PathBuf>>,

    /// Declaration hashes for incremental type checking (signature hashes per declaration)
    pub declaration_hashes: FxHashMap<PathBuf, FxHashMap<String, u64>>,

    /// Dependency graph for declarations: declaration -> list of dependent declarations
    /// Used to track which declarations depend on which, enabling precise invalidation
    #[serde(default)]
    pub declaration_dependencies: FxHashMap<PathBuf, FxHashMap<String, Vec<(PathBuf, String)>>>,
}

/// Entry for a single cached module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Canonical path to source file
    pub source_path: PathBuf,

    /// Blake3 hash of source content
    pub source_hash: String,

    /// Hash of the cached binary file (for integrity)
    pub cache_hash: String,

    /// Timestamp when cached (for diagnostics)
    pub cached_at: u64,

    /// List of direct dependencies (for invalidation)
    pub dependencies: Vec<PathBuf>,
}

impl CacheManifest {
    /// Create a new empty manifest with the given config hash
    pub fn new(config_hash: String) -> Self {
        Self {
            version: CACHE_VERSION,
            config_hash,
            modules: FxHashMap::default(),
            dependencies: FxHashMap::default(),
            declaration_hashes: FxHashMap::default(),
            declaration_dependencies: FxHashMap::default(),
        }
    }

    /// Check if manifest version matches current cache version
    pub fn is_version_compatible(&self) -> bool {
        self.version == CACHE_VERSION
    }

    /// Serialize manifest to binary format
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(CacheError::from)
    }

    /// Deserialize manifest from binary format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(CacheError::from)
    }

    /// Add or update a cache entry
    pub fn insert_entry(&mut self, path: PathBuf, entry: CacheEntry) {
        // Update dependencies graph
        self.dependencies
            .insert(path.clone(), entry.dependencies.clone());

        // Insert the cache entry
        self.modules.insert(path, entry);
    }

    /// Remove a cache entry and its dependency information
    pub fn remove_entry(&mut self, path: &PathBuf) {
        self.modules.remove(path);
        self.dependencies.remove(path);
    }

    /// Get a cache entry for a module
    pub fn get_entry(&self, path: &PathBuf) -> Option<&CacheEntry> {
        self.modules.get(path)
    }

    /// Clean up entries for files that no longer exist
    pub fn cleanup_stale_entries(&mut self, current_files: &[PathBuf]) {
        let current_set: std::collections::HashSet<_> = current_files.iter().collect();

        self.modules.retain(|path, _| current_set.contains(path));
        self.dependencies
            .retain(|path, _| current_set.contains(path));
        self.declaration_hashes
            .retain(|path, _| current_set.contains(path));
        self.declaration_dependencies
            .retain(|path, _| current_set.contains(path));
    }

    /// Update declaration hashes for a module
    pub fn update_declaration_hashes(
        &mut self,
        module_path: &PathBuf,
        hashes: FxHashMap<String, u64>,
    ) {
        self.declaration_hashes.insert(module_path.clone(), hashes);
    }

    /// Get declaration hashes for a module
    pub fn get_declaration_hashes(&self, module_path: &PathBuf) -> Option<&FxHashMap<String, u64>> {
        self.declaration_hashes.get(module_path)
    }

    /// Update declaration dependencies for a module
    pub fn update_declaration_dependencies(
        &mut self,
        module_path: &PathBuf,
        dependencies: FxHashMap<String, Vec<(PathBuf, String)>>,
    ) {
        self.declaration_dependencies
            .insert(module_path.clone(), dependencies);
    }

    /// Get declaration dependencies for a module
    pub fn get_declaration_dependencies(
        &self,
        module_path: &PathBuf,
    ) -> Option<&FxHashMap<String, Vec<(PathBuf, String)>>> {
        self.declaration_dependencies.get(module_path)
    }

    /// Check if declaration signatures have changed between old and new hashes
    ///
    /// Returns a list of declarations that have changed signatures
    pub fn get_changed_declarations(
        &self,
        module_path: &PathBuf,
        new_hashes: &FxHashMap<String, u64>,
    ) -> Vec<String> {
        let mut changed = Vec::new();

        if let Some(old_hashes) = self.declaration_hashes.get(module_path) {
            for (decl_name, new_hash) in new_hashes {
                if let Some(old_hash) = old_hashes.get(decl_name) {
                    if new_hash != old_hash {
                        changed.push(decl_name.clone());
                    }
                } else {
                    changed.push(decl_name.clone());
                }
            }
        } else {
            // New module - all declarations have changed
            changed.extend(new_hashes.keys().cloned());
        }

        changed
    }

    /// Get dependents of a changed declaration
    ///
    /// Returns a list of (module_path, declaration_name) pairs that depend on the changed declaration
    pub fn get_dependents_of_declaration(
        &self,
        module_path: &PathBuf,
        declaration_name: &str,
    ) -> Vec<(PathBuf, String)> {
        let mut dependents = Vec::new();

        // Check the same module for internal dependencies
        if let Some(deps) = self.declaration_dependencies.get(module_path) {
            if let Some(callers) = deps.get(declaration_name) {
                for (caller_module, caller_decl) in callers {
                    if caller_module == module_path {
                        dependents.push((caller_module.clone(), caller_decl.clone()));
                    }
                }
            }
        }

        // Check all modules for cross-module dependencies
        for (dep_module_path, dependencies) in &self.declaration_dependencies {
            if dep_module_path == module_path {
                continue;
            }
            for (callee, callers) in dependencies {
                if callee == declaration_name {
                    for (caller_module, caller_decl) in callers {
                        if caller_module == module_path {
                            dependents.push((caller_module.clone(), caller_decl.clone()));
                        }
                    }
                }
            }
        }

        dependents
    }
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        source_path: PathBuf,
        source_hash: String,
        cache_hash: String,
        dependencies: Vec<PathBuf>,
    ) -> Self {
        Self {
            source_path,
            source_hash,
            cache_hash,
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            dependencies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let mut manifest = CacheManifest::new("test_hash".to_string());

        let entry = CacheEntry::new(
            PathBuf::from("/test/file.tl"),
            "source_hash".to_string(),
            "cache_hash".to_string(),
            vec![PathBuf::from("/test/dep.tl")],
        );

        manifest.insert_entry(PathBuf::from("/test/file.tl"), entry);

        // Add declaration hashes
        let mut hashes = FxHashMap::default();
        hashes.insert("testFunc".to_string(), 12345);
        manifest.update_declaration_hashes(&PathBuf::from("/test/file.tl"), hashes);

        let bytes = manifest.to_bytes().unwrap();
        let deserialized = CacheManifest::from_bytes(&bytes).unwrap();

        assert_eq!(manifest.version, deserialized.version);
        assert_eq!(manifest.config_hash, deserialized.config_hash);
        assert_eq!(manifest.modules.len(), deserialized.modules.len());
        assert_eq!(
            manifest.declaration_hashes.len(),
            deserialized.declaration_hashes.len()
        );
    }

    #[test]
    fn test_manifest_version_compatibility() {
        let manifest = CacheManifest::new("test".to_string());
        assert!(manifest.is_version_compatible());
    }

    #[test]
    fn test_cleanup_stale_entries() {
        let mut manifest = CacheManifest::new("test".to_string());

        let entry1 = CacheEntry::new(
            PathBuf::from("/test/file1.tl"),
            "hash1".to_string(),
            "cache1".to_string(),
            vec![],
        );

        let entry2 = CacheEntry::new(
            PathBuf::from("/test/file2.tl"),
            "hash2".to_string(),
            "cache2".to_string(),
            vec![],
        );

        manifest.insert_entry(PathBuf::from("/test/file1.tl"), entry1);
        manifest.insert_entry(PathBuf::from("/test/file2.tl"), entry2);

        // Add declaration hashes for both files
        let mut hashes1 = FxHashMap::default();
        hashes1.insert("func1".to_string(), 111);
        manifest.update_declaration_hashes(&PathBuf::from("/test/file1.tl"), hashes1);

        let mut hashes2 = FxHashMap::default();
        hashes2.insert("func2".to_string(), 222);
        manifest.update_declaration_hashes(&PathBuf::from("/test/file2.tl"), hashes2);

        // Only keep file1
        manifest.cleanup_stale_entries(&[PathBuf::from("/test/file1.tl")]);

        assert_eq!(manifest.modules.len(), 1);
        assert!(manifest
            .modules
            .contains_key(&PathBuf::from("/test/file1.tl")));
        assert!(!manifest
            .modules
            .contains_key(&PathBuf::from("/test/file2.tl")));

        // Declaration hashes should also be cleaned up
        assert_eq!(manifest.declaration_hashes.len(), 1);
        assert!(manifest
            .declaration_hashes
            .contains_key(&PathBuf::from("/test/file1.tl")));
    }

    #[test]
    fn test_declaration_hash_updates() {
        let mut manifest = CacheManifest::new("test".to_string());

        let mut hashes = FxHashMap::default();
        hashes.insert("testFunc".to_string(), 12345);
        manifest.update_declaration_hashes(&PathBuf::from("/test/file.tl"), hashes.clone());

        let retrieved = manifest.get_declaration_hashes(&PathBuf::from("/test/file.tl"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get("testFunc"), Some(&12345));
    }

    #[test]
    fn test_changed_declarations() {
        let mut manifest = CacheManifest::new("test".to_string());

        // Add old hashes
        let mut old_hashes = FxHashMap::default();
        old_hashes.insert("func1".to_string(), 100);
        old_hashes.insert("func2".to_string(), 200);
        manifest.update_declaration_hashes(&PathBuf::from("/test/file.tl"), old_hashes);

        // New hashes - func1 changed, func2 same, func3 new
        let mut new_hashes = FxHashMap::default();
        new_hashes.insert("func1".to_string(), 101); // Changed
        new_hashes.insert("func2".to_string(), 200); // Same
        new_hashes.insert("func3".to_string(), 300); // New

        let changed =
            manifest.get_changed_declarations(&PathBuf::from("/test/file.tl"), &new_hashes);

        assert_eq!(changed.len(), 2);
        assert!(changed.contains(&"func1".to_string()));
        assert!(changed.contains(&"func3".to_string()));
    }
}
