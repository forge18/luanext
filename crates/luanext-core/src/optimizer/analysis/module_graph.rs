use luanext_parser::ast::statement::{
    ExportDeclaration, ExportKind, ImportClause, ImportDeclaration, Statement,
};
use luanext_parser::string_interner::StringInterner;
use luanext_typechecker::module_resolver::registry::ModuleRegistry;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Whole-program module dependency graph with reachability info
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    /// Map from module path to its metadata
    pub modules: FxHashMap<PathBuf, ModuleNode>,

    /// Entry points (main files, explicitly compiled files)
    pub entry_points: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub path: PathBuf,

    /// Symbols this module exports
    pub exports: FxHashMap<String, ExportInfo>,

    /// Symbols this module imports (and from where)
    pub imports: FxHashMap<String, ImportInfo>,

    /// Re-exports (export * from, export { x } from)
    pub re_exports: Vec<ReExportInfo>,

    /// Whether this module is reachable from any entry point
    pub is_reachable: bool,
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub name: String,
    pub is_type_only: bool,
    pub is_default: bool,
    /// Tracks if this export is imported by any other module
    pub is_used: bool,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub name: String,
    pub source_module: PathBuf,
    pub source_symbol: String,
    pub is_type_only: bool,
    /// Tracks if this imported symbol is actually referenced in code
    pub is_referenced: bool,
}

#[derive(Debug, Clone)]
pub struct ReExportInfo {
    pub source_module: PathBuf,
    pub specifiers: ReExportKind,
}

#[derive(Debug, Clone)]
pub enum ReExportKind {
    /// export * from './foo'
    All,
    /// export { x as y } from './foo'
    Named(Vec<(String, String)>),
}

impl ModuleGraph {
    /// Build module graph from all programs and their metadata
    pub fn build(
        modules: &[(PathBuf, &[Statement])],
        interner: Arc<StringInterner>,
        _registry: &ModuleRegistry,
        entry_points: &[PathBuf],
    ) -> Self {
        let mut graph = ModuleGraph {
            modules: FxHashMap::default(),
            entry_points: entry_points.iter().cloned().collect(),
        };

        // Phase 1: Extract export/import info from each module
        for (path, statements) in modules {
            graph.add_module(path, statements, &interner);
        }

        // Phase 2: Mark which imports are actually referenced in code
        for (path, statements) in modules {
            graph.mark_import_references(path, statements, &interner);
        }

        // Phase 3: Resolve relative source paths to canonical module paths
        graph.resolve_source_paths();

        // Phase 4: Compute reachability from entry points
        graph.compute_reachability();

        // Phase 5: Mark used exports
        graph.mark_usage();

        graph
    }

    /// Extract exports and imports from a module's AST
    fn add_module(&mut self, path: &Path, statements: &[Statement], interner: &StringInterner) {
        let mut node = ModuleNode {
            path: path.to_path_buf(),
            exports: FxHashMap::default(),
            imports: FxHashMap::default(),
            re_exports: Vec::new(),
            is_reachable: false,
        };

        // Scan statements for exports and imports
        for statement in statements {
            match statement {
                Statement::Export(export) => {
                    self.extract_exports_from_declaration(export, &mut node, interner);
                }
                Statement::Import(import) => {
                    self.extract_imports_from_declaration(import, &mut node, interner);
                }
                _ => {}
            }
        }

        self.modules.insert(path.to_path_buf(), node);
    }

    /// Extract export information from an export declaration
    fn extract_exports_from_declaration(
        &self,
        export: &ExportDeclaration,
        node: &mut ModuleNode,
        interner: &StringInterner,
    ) {
        match &export.kind {
            ExportKind::Declaration(stmt) => {
                // export function foo() or export const x = 1
                match stmt {
                    Statement::Function(func) => {
                        let name = interner.resolve(func.name.node).to_string();
                        node.exports.insert(
                            name.clone(),
                            ExportInfo {
                                name,
                                is_type_only: false,
                                is_default: false,
                                is_used: false,
                            },
                        );
                    }
                    Statement::Variable(var) => {
                        // Extract name from pattern
                        if let Some(name) = self.extract_name_from_pattern(&var.pattern, interner) {
                            node.exports.insert(
                                name.clone(),
                                ExportInfo {
                                    name,
                                    is_type_only: false,
                                    is_default: false,
                                    is_used: false,
                                },
                            );
                        }
                    }
                    Statement::Class(class) => {
                        let name = interner.resolve(class.name.node).to_string();
                        node.exports.insert(
                            name.clone(),
                            ExportInfo {
                                name,
                                is_type_only: false,
                                is_default: false,
                                is_used: false,
                            },
                        );
                    }
                    _ => {}
                }
            }
            ExportKind::Named {
                specifiers,
                source,
                is_type_only,
            } => {
                if let Some(source_path) = source {
                    // This is a re-export: export { x } from './foo'
                    let named_specs: Vec<(String, String)> = specifiers
                        .iter()
                        .map(|spec| {
                            let local_name = interner.resolve(spec.local.node).to_string();
                            let exported_name = spec
                                .exported
                                .as_ref()
                                .map(|e| interner.resolve(e.node).to_string())
                                .unwrap_or_else(|| local_name.clone());
                            (local_name, exported_name)
                        })
                        .collect();

                    node.re_exports.push(ReExportInfo {
                        source_module: PathBuf::from(source_path),
                        specifiers: ReExportKind::Named(named_specs),
                    });
                } else {
                    // Regular named export: export { x, y }
                    for specifier in specifiers.iter() {
                        let local_name = interner.resolve(specifier.local.node).to_string();
                        let export_name = specifier
                            .exported
                            .as_ref()
                            .map(|e| interner.resolve(e.node).to_string())
                            .unwrap_or_else(|| local_name);

                        node.exports.insert(
                            export_name.clone(),
                            ExportInfo {
                                name: export_name,
                                is_type_only: *is_type_only,
                                is_default: false,
                                is_used: false,
                            },
                        );
                    }
                }
            }
            ExportKind::Default(_) => {
                node.exports.insert(
                    "default".to_string(),
                    ExportInfo {
                        name: "default".to_string(),
                        is_type_only: false,
                        is_default: true,
                        is_used: false,
                    },
                );
            }
            ExportKind::All {
                source,
                is_type_only: _,
            } => {
                // export * from './foo'
                node.re_exports.push(ReExportInfo {
                    source_module: PathBuf::from(source),
                    specifiers: ReExportKind::All,
                });
            }
        }
    }

    /// Extract import information from an import declaration
    fn extract_imports_from_declaration(
        &self,
        import: &ImportDeclaration,
        node: &mut ModuleNode,
        interner: &StringInterner,
    ) {
        let source_path = PathBuf::from(&import.source);

        match &import.clause {
            ImportClause::Default(id) => {
                let name = interner.resolve(id.node).to_string();
                node.imports.insert(
                    name.clone(),
                    ImportInfo {
                        name,
                        source_module: source_path,
                        source_symbol: "default".to_string(),
                        is_type_only: false,
                        is_referenced: false,
                    },
                );
            }
            ImportClause::Named(specifiers) => {
                for specifier in specifiers.iter() {
                    let source_symbol = interner.resolve(specifier.imported.node).to_string();
                    let local_name = specifier
                        .local
                        .as_ref()
                        .map(|l| interner.resolve(l.node).to_string())
                        .unwrap_or_else(|| source_symbol.clone());

                    node.imports.insert(
                        local_name.clone(),
                        ImportInfo {
                            name: local_name,
                            source_module: source_path.clone(),
                            source_symbol,
                            is_type_only: false,
                            is_referenced: false,
                        },
                    );
                }
            }
            ImportClause::Namespace(id) => {
                let name = interner.resolve(id.node).to_string();
                node.imports.insert(
                    name.clone(),
                    ImportInfo {
                        name,
                        source_module: source_path,
                        source_symbol: "*".to_string(),
                        is_type_only: false,
                        is_referenced: false,
                    },
                );
            }
            ImportClause::TypeOnly(_) => {
                // Type-only imports don't create runtime dependencies
                // They're erased at codegen, so skip them
            }
            ImportClause::Mixed { default, named } => {
                // Default import
                let default_name = interner.resolve(default.node).to_string();
                node.imports.insert(
                    default_name.clone(),
                    ImportInfo {
                        name: default_name,
                        source_module: source_path.clone(),
                        source_symbol: "default".to_string(),
                        is_type_only: false,
                        is_referenced: false,
                    },
                );

                // Named imports
                for specifier in named.iter() {
                    let source_symbol = interner.resolve(specifier.imported.node).to_string();
                    let local_name = specifier
                        .local
                        .as_ref()
                        .map(|l| interner.resolve(l.node).to_string())
                        .unwrap_or_else(|| source_symbol.clone());

                    node.imports.insert(
                        local_name.clone(),
                        ImportInfo {
                            name: local_name,
                            source_module: source_path.clone(),
                            source_symbol,
                            is_type_only: false,
                            is_referenced: false,
                        },
                    );
                }
            }
        }
    }

    /// Helper to extract simple identifier name from pattern
    fn extract_name_from_pattern(
        &self,
        pattern: &luanext_parser::ast::pattern::Pattern,
        interner: &StringInterner,
    ) -> Option<String> {
        use luanext_parser::ast::pattern::Pattern;

        match pattern {
            Pattern::Identifier(id) => Some(interner.resolve(id.node).to_string()),
            _ => None, // For now, skip destructuring patterns
        }
    }

    /// Resolve relative source paths in imports and re-exports to canonical module paths.
    ///
    /// After Phase 1 (add_module), `ImportInfo.source_module` and `ReExportInfo.source_module`
    /// contain raw relative strings like `PathBuf::from("./b")`. This method resolves them
    /// to the canonical paths used as keys in `self.modules`, enabling correct lookups in
    /// `resolve_re_export_chain()`, `compute_reachability()`, and `mark_usage()`.
    fn resolve_source_paths(&mut self) {
        let known_modules: Vec<PathBuf> = self.modules.keys().cloned().collect();

        // Collect all resolutions first to avoid borrow issues
        let mut import_resolutions: Vec<(PathBuf, String, PathBuf)> = Vec::new();
        let mut reexport_resolutions: Vec<(PathBuf, usize, PathBuf)> = Vec::new();

        for (module_path, node) in &self.modules {
            let parent = module_path
                .parent()
                .unwrap_or(module_path.as_path())
                .to_path_buf();

            // Resolve import source paths
            for (name, import_info) in &node.imports {
                let source_str = import_info.source_module.to_string_lossy();
                if let Some(resolved) =
                    resolve_relative_source(&parent, &source_str, &known_modules)
                {
                    import_resolutions.push((module_path.clone(), name.clone(), resolved));
                }
            }

            // Resolve re-export source paths
            for (idx, re_export) in node.re_exports.iter().enumerate() {
                let source_str = re_export.source_module.to_string_lossy();
                if let Some(resolved) =
                    resolve_relative_source(&parent, &source_str, &known_modules)
                {
                    reexport_resolutions.push((module_path.clone(), idx, resolved));
                }
            }
        }

        // Apply import resolutions
        for (module_path, import_name, resolved) in import_resolutions {
            if let Some(node) = self.modules.get_mut(&module_path) {
                if let Some(import_info) = node.imports.get_mut(&import_name) {
                    import_info.source_module = resolved;
                }
            }
        }

        // Apply re-export resolutions
        for (module_path, idx, resolved) in reexport_resolutions {
            if let Some(node) = self.modules.get_mut(&module_path) {
                if let Some(re_export) = node.re_exports.get_mut(idx) {
                    re_export.source_module = resolved;
                }
            }
        }
    }

    /// DFS from entry points to mark reachable modules
    fn compute_reachability(&mut self) {
        let mut visited = FxHashSet::default();
        let entry_points: Vec<PathBuf> = self.entry_points.iter().cloned().collect();

        for entry in &entry_points {
            self.mark_reachable_recursive(entry, &mut visited);
        }
    }

    fn mark_reachable_recursive(&mut self, path: &Path, visited: &mut FxHashSet<PathBuf>) {
        if visited.contains(path) {
            return;
        }
        visited.insert(path.to_path_buf());

        // Collect all dependent modules before recursing (to avoid borrow issues)
        let dependent_modules: Vec<PathBuf> = if let Some(node) = self.modules.get(path) {
            let mut deps = Vec::new();

            // Collect imported modules (skip type-only imports)
            for imp in node.imports.values() {
                if !imp.is_type_only {
                    deps.push(imp.source_module.clone());
                }
            }

            // Collect re-exported modules
            for re in &node.re_exports {
                deps.push(re.source_module.clone());
            }

            deps
        } else {
            Vec::new()
        };

        // Now mark this module as reachable (after collecting dependencies)
        if let Some(node) = self.modules.get_mut(path) {
            node.is_reachable = true;
        }

        // Recursively mark imported and re-exported modules as reachable
        for source in dependent_modules {
            self.mark_reachable_recursive(&source, visited);
        }
    }

    /// Mark which exports are actually used (imported elsewhere)
    fn mark_usage(&mut self) {
        // Build reverse map: which exports are imported by which modules
        let mut export_usage: FxHashMap<(PathBuf, String), usize> = FxHashMap::default();

        for node in self.modules.values() {
            for import in node.imports.values() {
                let key = (import.source_module.clone(), import.source_symbol.clone());
                *export_usage.entry(key).or_insert(0) += 1;
            }
        }

        // Mark exports as used if they're imported
        for node in self.modules.values_mut() {
            for export in node.exports.values_mut() {
                let key = (node.path.clone(), export.name.clone());
                if export_usage.contains_key(&key) {
                    export.is_used = true;
                }
            }
        }

        // Note: is_referenced tracking requires AST traversal which needs
        // the original statements. This should be done during add_module()
        // by passing statements to a helper method. For now, we conservatively
        // mark all imports as potentially referenced.
    }

    /// Track which imported symbols are actually referenced in the module's code
    /// This should be called during add_module() with the statements
    pub fn mark_import_references(
        &mut self,
        module_path: &Path,
        statements: &[Statement],
        interner: &StringInterner,
    ) {
        // Collect all imported names to look for (avoiding borrow issues)
        let import_names: FxHashSet<String> = if let Some(node) = self.modules.get(module_path) {
            node.imports.keys().cloned().collect()
        } else {
            return;
        };

        // Traverse statements looking for identifier references
        let referenced = self.find_identifier_references(statements, &import_names, interner);

        // Mark imports as referenced if found
        if let Some(node) = self.modules.get_mut(module_path) {
            for (name, import_info) in node.imports.iter_mut() {
                if referenced.contains(name) {
                    import_info.is_referenced = true;
                }
            }
        }
    }

    /// Recursively traverse statements to find identifier references
    fn find_identifier_references(
        &self,
        statements: &[Statement],
        import_names: &FxHashSet<String>,
        interner: &StringInterner,
    ) -> FxHashSet<String> {
        let mut referenced = FxHashSet::default();

        for stmt in statements {
            self.find_references_in_statement(stmt, import_names, interner, &mut referenced);
        }

        referenced
    }

    /// Find references in a single statement
    fn find_references_in_statement(
        &self,
        stmt: &Statement,
        import_names: &FxHashSet<String>,
        interner: &StringInterner,
        referenced: &mut FxHashSet<String>,
    ) {
        match stmt {
            Statement::Variable(var) => {
                self.find_references_in_expression(
                    &var.initializer,
                    import_names,
                    interner,
                    referenced,
                );
            }
            Statement::Function(func) => {
                for stmt in func.body.statements {
                    self.find_references_in_statement(stmt, import_names, interner, referenced);
                }
            }
            Statement::Expression(expr) => {
                self.find_references_in_expression(expr, import_names, interner, referenced);
            }
            Statement::Return(ret) => {
                for expr in ret.values.iter() {
                    self.find_references_in_expression(expr, import_names, interner, referenced);
                }
            }
            Statement::If(if_stmt) => {
                self.find_references_in_expression(
                    &if_stmt.condition,
                    import_names,
                    interner,
                    referenced,
                );
                for stmt in if_stmt.then_block.statements {
                    self.find_references_in_statement(stmt, import_names, interner, referenced);
                }
                for else_if in if_stmt.else_ifs.iter() {
                    self.find_references_in_expression(
                        &else_if.condition,
                        import_names,
                        interner,
                        referenced,
                    );
                    for stmt in else_if.block.statements {
                        self.find_references_in_statement(stmt, import_names, interner, referenced);
                    }
                }
                if let Some(else_block) = &if_stmt.else_block {
                    for stmt in else_block.statements {
                        self.find_references_in_statement(stmt, import_names, interner, referenced);
                    }
                }
            }
            Statement::While(while_stmt) => {
                self.find_references_in_expression(
                    &while_stmt.condition,
                    import_names,
                    interner,
                    referenced,
                );
                for stmt in while_stmt.body.statements {
                    self.find_references_in_statement(stmt, import_names, interner, referenced);
                }
            }
            Statement::For(for_stmt) => {
                // Handle both for-loop variants
                match for_stmt {
                    luanext_parser::ast::statement::ForStatement::Numeric(numeric) => {
                        self.find_references_in_expression(
                            &numeric.start,
                            import_names,
                            interner,
                            referenced,
                        );
                        self.find_references_in_expression(
                            &numeric.end,
                            import_names,
                            interner,
                            referenced,
                        );
                        if let Some(step) = &numeric.step {
                            self.find_references_in_expression(
                                step,
                                import_names,
                                interner,
                                referenced,
                            );
                        }
                        for stmt in numeric.body.statements {
                            self.find_references_in_statement(
                                stmt,
                                import_names,
                                interner,
                                referenced,
                            );
                        }
                    }
                    luanext_parser::ast::statement::ForStatement::Generic(generic) => {
                        for expr in generic.iterators.iter() {
                            self.find_references_in_expression(
                                expr,
                                import_names,
                                interner,
                                referenced,
                            );
                        }
                        for stmt in generic.body.statements {
                            self.find_references_in_statement(
                                stmt,
                                import_names,
                                interner,
                                referenced,
                            );
                        }
                    }
                }
            }
            Statement::Block(block) => {
                for stmt in block.statements {
                    self.find_references_in_statement(stmt, import_names, interner, referenced);
                }
            }
            _ => {
                // Other statement types - skip for now
            }
        }
    }

    /// Find references in an expression
    fn find_references_in_expression(
        &self,
        expr: &luanext_parser::ast::expression::Expression,
        import_names: &FxHashSet<String>,
        interner: &StringInterner,
        referenced: &mut FxHashSet<String>,
    ) {
        use luanext_parser::ast::expression::ExpressionKind;

        match &expr.kind {
            ExpressionKind::Identifier(id) => {
                let name = interner.resolve(*id).to_string();
                if import_names.contains(&name) {
                    referenced.insert(name);
                }
            }
            ExpressionKind::Call(callee, arguments, _) => {
                self.find_references_in_expression(callee, import_names, interner, referenced);
                for arg in arguments.iter() {
                    self.find_references_in_expression(
                        &arg.value,
                        import_names,
                        interner,
                        referenced,
                    );
                }
            }
            ExpressionKind::Member(object, _) => {
                self.find_references_in_expression(object, import_names, interner, referenced);
            }
            ExpressionKind::Binary(_, left, right) => {
                self.find_references_in_expression(left, import_names, interner, referenced);
                self.find_references_in_expression(right, import_names, interner, referenced);
            }
            ExpressionKind::Unary(_, operand) => {
                self.find_references_in_expression(operand, import_names, interner, referenced);
            }
            ExpressionKind::Array(elements) => {
                for elem in elements.iter() {
                    if let luanext_parser::ast::expression::ArrayElement::Expression(e) = elem {
                        self.find_references_in_expression(e, import_names, interner, referenced);
                    }
                }
            }
            ExpressionKind::Object(props) => {
                for prop in props.iter() {
                    if let luanext_parser::ast::expression::ObjectProperty::Property {
                        value, ..
                    } = prop
                    {
                        self.find_references_in_expression(
                            value,
                            import_names,
                            interner,
                            referenced,
                        );
                    }
                }
            }
            _ => {
                // Other expression kinds - best effort, skip complex cases
            }
        }
    }

    /// Flatten re-export chains to find the original export source
    /// Returns (original_module_path, original_symbol_name)
    pub fn resolve_re_export_chain(
        &self,
        module: &Path,
        symbol: &str,
    ) -> Option<(PathBuf, String)> {
        let mut visited = FxHashSet::default();
        self.resolve_re_export_chain_recursive(module, symbol, &mut visited, 0)
    }

    fn resolve_re_export_chain_recursive(
        &self,
        module: &Path,
        symbol: &str,
        visited: &mut FxHashSet<PathBuf>,
        depth: usize,
    ) -> Option<(PathBuf, String)> {
        const MAX_DEPTH: usize = 10;

        // Prevent infinite recursion
        if depth > MAX_DEPTH {
            return None;
        }

        // Detect cycles
        if visited.contains(module) {
            return None;
        }
        visited.insert(module.to_path_buf());

        let node = self.modules.get(module)?;

        // Check if this module directly exports the symbol
        if node.exports.contains_key(symbol) {
            return Some((module.to_path_buf(), symbol.to_string()));
        }

        // Check re-exports
        for re_export in &node.re_exports {
            match &re_export.specifiers {
                ReExportKind::All => {
                    // export * from './source' - try to resolve in source module
                    if let Some(result) = self.resolve_re_export_chain_recursive(
                        &re_export.source_module,
                        symbol,
                        visited,
                        depth + 1,
                    ) {
                        return Some(result);
                    }
                }
                ReExportKind::Named(specs) => {
                    // export { x as y } from './source'
                    for (local, exported) in specs {
                        if exported == symbol {
                            // This re-export matches our symbol
                            // Recursively resolve the source
                            return self.resolve_re_export_chain_recursive(
                                &re_export.source_module,
                                local,
                                visited,
                                depth + 1,
                            );
                        }
                    }
                }
            }
        }

        None
    }
}

/// Resolve a relative import source string (e.g., `./b`, `../utils`) to a canonical
/// module path from the set of known modules.
///
/// Tries the source path with common extensions (`.luax`, `.d.luax`, `.lua`) and
/// directory index patterns (`index.luax`).
pub fn resolve_relative_source(
    from_dir: &Path,
    source: &str,
    known_modules: &[PathBuf],
) -> Option<PathBuf> {
    // Only resolve relative paths
    if !source.starts_with("./") && !source.starts_with("../") {
        return None;
    }

    let target = normalize_path(&from_dir.join(source));

    // Try exact match first
    for known in known_modules {
        if paths_equal(known, &target) {
            return Some(known.clone());
        }
    }

    // Try with extensions
    for ext in &["luax", "d.luax", "lua"] {
        let with_ext = target.with_extension(ext);
        for known in known_modules {
            if paths_equal(known, &with_ext) {
                return Some(known.clone());
            }
        }
    }

    // Try directory index patterns
    let index_path = target.join("index.luax");
    for known in known_modules {
        if paths_equal(known, &index_path) {
            return Some(known.clone());
        }
    }

    None
}

/// Normalize a path by resolving `.` and `..` components without filesystem access.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {} // skip "."
            std::path::Component::ParentDir => {
                // Pop the last normal component if possible
                if let Some(last) = components.last() {
                    if *last != std::path::Component::ParentDir {
                        components.pop();
                    } else {
                        components.push(component);
                    }
                } else {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

/// Compare two paths for equality, normalizing both first.
fn paths_equal(a: &Path, b: &Path) -> bool {
    normalize_path(a) == normalize_path(b)
}

/// Compute a relative require path from one module to another.
///
/// Given `from = /project/src/a.luax` and `to = /project/src/lib/c.luax`,
/// returns `./lib/c` (no extension, with `./` prefix).
pub fn compute_relative_require_path(from: &Path, to: &Path) -> String {
    let from_dir = from.parent().unwrap_or(from);
    let to_normalized = normalize_path(to);
    let from_dir_normalized = normalize_path(from_dir);

    // Try to compute relative path
    if let Some(rel) = pathdiff_relative(&from_dir_normalized, &to_normalized) {
        let rel_str = rel.to_string_lossy();
        // Strip extension
        let without_ext = strip_module_extension(&rel_str);
        // Ensure ./ prefix
        if without_ext.starts_with("./") || without_ext.starts_with("../") {
            without_ext
        } else {
            format!("./{without_ext}")
        }
    } else {
        // Fallback: use the target path as-is without extension
        let to_str = to_normalized.to_string_lossy();
        strip_module_extension(&to_str)
    }
}

/// Compute relative path from a directory to a target file.
fn pathdiff_relative(from_dir: &Path, to: &Path) -> Option<PathBuf> {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();

    // Find common prefix length
    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common_len == 0 {
        return None;
    }

    let mut result = PathBuf::new();

    // Add "../" for each remaining component in from_dir
    let ups = from_components.len() - common_len;
    for _ in 0..ups {
        result.push("..");
    }

    // Add remaining components from target
    for component in &to_components[common_len..] {
        result.push(component);
    }

    // If no ups needed, prefix with "./"
    if ups == 0 {
        let mut prefixed = PathBuf::from(".");
        prefixed.push(result);
        result = prefixed;
    }

    Some(result)
}

/// Strip module file extensions (.luax, .d.luax, .lua) from a path string.
fn strip_module_extension(path: &str) -> String {
    for ext in &[".luax", ".d.luax", ".lua"] {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    path.to_string()
}
