use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Generate a LuaNext module with realistic cross-file type dependencies
fn generate_module_with_types(name: &str, imports: &[&str], exports_count: usize) -> String {
    let mut code = format!("-- Module: {}\n\n", name);

    // Add imports
    for import in imports {
        code.push_str(&format!(
            "import type {{ Data{}, Process{} }} from \"./module_{}\"\n",
            import, import, import
        ));
    }
    code.push('\n');

    // Add interfaces
    for i in 0..exports_count {
        code.push_str(&format!(
            r#"interface Data{}{} {{
    id: number
    name: string
    value: number
    process(input: number): number
}}

"#,
            name, i
        ));
    }

    // Add classes
    for i in 0..exports_count {
        code.push_str(&format!(
            r#"class Process{}{} {{
    private data: Data{}{}

    constructor(initial: Data{}{})
        self.data = initial
    end

    execute(n: number): number
        return self.data.process(n * 2)
    end
}}

"#,
            name, i, name, i, name, i
        ));
    }

    // Export types
    code.push_str("export type {\n");
    for i in 0..exports_count {
        code.push_str(&format!("    Data{}{},\n", name, i));
        code.push_str(&format!("    Process{}{},\n", name, i));
    }
    code.push_str("}\n");

    code
}

/// Generate a re-export module chain
fn generate_reexport_module(name: &str, source_modules: &[&str]) -> String {
    let mut code = format!("-- Re-export module: {}\n\n", name);

    for module in source_modules {
        code.push_str(&format!("export type * from \"./module_{}\"\n", module));
    }

    code
}

/// Generate a large project with cross-file type dependencies
fn generate_large_project(
    module_count: usize,
    types_per_module: usize,
    reexport_chain_depth: usize,
) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate modules in layers to create cross-file dependencies
    for i in 0..module_count {
        let module_name = format!("module_{}", i);

        // Each module imports from the previous layer
        let mut imports = vec![];
        if i > 0 && i <= 5 {
            // First few modules import from module_0
            imports.push("0");
        } else if i > 5 && i <= 20 {
            // Next layer imports from first layer
            let prev = i - 5;
            imports.push(Box::leak(prev.to_string().into_boxed_str()) as &str);
        } else if i > 20 {
            // Later modules import from multiple previous modules
            let prev1 = i - 10;
            let prev2 = i - 20;
            imports.push(Box::leak(prev1.to_string().into_boxed_str()) as &str);
            imports.push(Box::leak(prev2.to_string().into_boxed_str()) as &str);
        }

        let module_content = generate_module_with_types(&module_name, &imports, types_per_module);
        let module_path = temp_dir.path().join(format!("{}.luax", module_name));
        fs::write(module_path, module_content).expect("Failed to write module");
    }

    // Create re-export chain
    for depth in 1..=reexport_chain_depth {
        let reexport_name = format!("reexport_{}", depth);
        let sources: Vec<String> = if depth == 1 {
            // First level re-exports from base modules
            (0..5).map(|i| i.to_string()).collect()
        } else {
            // Deeper levels re-export from previous re-export level
            vec![format!("reexport_{}", depth - 1)]
        };

        let source_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
        let reexport_content = generate_reexport_module(&reexport_name, &source_refs);
        let reexport_path = temp_dir.path().join(format!("{}.luax", reexport_name));
        fs::write(reexport_path, reexport_content).expect("Failed to write re-export");
    }

    // Create main module that imports everything
    let mut main_content = String::from("-- Main module with all imports\n\n");

    // Import from re-export chain
    if reexport_chain_depth > 0 {
        main_content.push_str(&format!(
            "import type {{ Data0_0, Process0_0 }} from \"./reexport_{}\"\n\n",
            reexport_chain_depth
        ));
    }

    // Import from regular modules
    for i in 0..5.min(module_count) {
        main_content.push_str(&format!(
            "import type {{ Data{}_0 }} from \"./module_{}\"\n",
            i, i
        ));
    }

    main_content.push('\n');
    main_content.push_str("function main(): void\n");
    main_content.push_str("    print(\"Compiled successfully\")\n");
    main_content.push_str("end\n");

    let main_path = temp_dir.path().join("main.luax");
    fs::write(main_path, main_content).expect("Failed to write main module");

    temp_dir
}

/// Compile project and measure time
fn compile_project(project_path: &Path, use_cache: bool) -> Result<(), String> {
    let binary_path = env!("CARGO_BIN_EXE_luanext");
    let mut cmd = Command::new(binary_path);

    cmd.arg("compile")
        .arg(project_path.join("main.luax"))
        .arg("--no-emit");

    if !use_cache {
        cmd.arg("--no-cache");
    }

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Benchmark: Large project compilation (100+ files)
/// Target: <5 seconds for clean build
fn benchmark_large_project_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_project_compilation");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    // Test with increasing project sizes
    for module_count in [50, 100, 150, 200] {
        let project = generate_large_project(module_count, 3, 0);
        let project_path = project.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::new("clean_build", module_count),
            &project_path,
            |b, path| {
                b.iter(|| {
                    compile_project(path, false).expect("Compilation failed");
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Incremental compilation with cross-file changes
/// Target: <1 second for single file change
fn benchmark_incremental_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_compilation");
    group.sample_size(10);

    let project = generate_large_project(100, 3, 0);
    let project_path = project.path().to_path_buf();

    // First compile to populate cache
    compile_project(&project_path, true).expect("Initial compilation failed");

    // Modify a single file
    let module_path = project_path.join("module_50.luax");
    let original_content = fs::read_to_string(&module_path).expect("Failed to read module");

    group.bench_function("single_file_change", |b| {
        b.iter(|| {
            // Modify the file
            let modified_content = original_content.clone() + "\n-- Modified\n";
            fs::write(&module_path, &modified_content).expect("Failed to write module");

            // Compile with cache
            compile_project(&project_path, true).expect("Incremental compilation failed");

            // Restore original content for next iteration
            fs::write(&module_path, &original_content).expect("Failed to restore module");
        })
    });

    group.finish();
}

/// Benchmark: Re-export chain resolution performance
/// Target: No significant degradation with deep chains
fn benchmark_reexport_chains(c: &mut Criterion) {
    let mut group = c.benchmark_group("reexport_chains");
    group.sample_size(10);

    // Test with varying re-export chain depths
    for depth in [1, 3, 5, 7, 10] {
        let project = generate_large_project(50, 2, depth);
        let project_path = project.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::new("chain_depth", depth),
            &project_path,
            |b, path| {
                b.iter(|| {
                    compile_project(path, false).expect("Compilation failed");
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Type resolution with many cross-file references
fn benchmark_cross_file_type_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_file_type_resolution");
    group.sample_size(10);

    // Test with varying numbers of types per module
    for types_per_module in [1, 3, 5, 10] {
        let project = generate_large_project(50, types_per_module, 2);
        let project_path = project.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::new("types_per_module", types_per_module),
            &project_path,
            |b, path| {
                b.iter(|| {
                    compile_project(path, false).expect("Compilation failed");
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_large_project_compilation,
    benchmark_incremental_compilation,
    benchmark_reexport_chains,
    benchmark_cross_file_type_resolution
);
criterion_main!(benches);
