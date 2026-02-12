use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Generate a module with specified complexity
fn generate_cached_module(name: &str, function_count: usize, type_count: usize) -> String {
    let mut code = format!("-- Cached module: {}\n\n", name);

    // Add interfaces
    for i in 0..type_count {
        code.push_str(&format!(
            r#"interface ICache{} {{
    get(key: string): number
    set(key: string, value: number): void
    clear(): void
}}

"#,
            i
        ));
    }

    // Add type aliases
    for i in 0..type_count {
        code.push_str(&format!(
            "type CacheEntry{} = {{ key: string, value: number, timestamp: number }}\n\n",
            i
        ));
    }

    // Add functions
    for i in 0..function_count {
        code.push_str(&format!(
            r#"function process{}(input: number): number
    const x = input * 2
    const y = x + 10
    return y
end

"#,
            i
        ));
    }

    // Export types
    code.push_str("export type {\n");
    for i in 0..type_count {
        code.push_str(&format!("    ICache{},\n", i));
        code.push_str(&format!("    CacheEntry{},\n", i));
    }
    code.push_str("}\n");

    code
}

/// Generate a project for cache testing
fn generate_cache_test_project(module_count: usize, functions_per_module: usize) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    for i in 0..module_count {
        let module_name = format!("module_{}", i);
        let module_content = generate_cached_module(&module_name, functions_per_module, 3);
        let module_path = temp_dir.path().join(format!("{}.luax", module_name));
        fs::write(module_path, module_content).expect("Failed to write module");
    }

    // Create main module
    let mut main_content = String::from("-- Main module\n\n");
    for i in 0..5.min(module_count) {
        main_content.push_str(&format!(
            "import type {{ ICache0 }} from \"./module_{}\"\n",
            i
        ));
    }
    main_content.push_str("\nfunction main(): void\n");
    main_content.push_str("    print(\"Cache test\")\n");
    main_content.push_str("end\n");

    let main_path = temp_dir.path().join("main.luax");
    fs::write(main_path, main_content).expect("Failed to write main");

    temp_dir
}

/// Compile with or without cache
fn compile_with_cache(project_path: &PathBuf, use_cache: bool) -> Result<std::time::Duration, String> {
    let binary_path = env!("CARGO_BIN_EXE_luanext");
    let mut cmd = Command::new(binary_path);

    cmd.arg("compile")
        .arg(project_path.join("main.luax"))
        .arg("--no-emit");

    if !use_cache {
        cmd.arg("--no-cache");
    }

    let start = std::time::Instant::now();
    let output = cmd.output().map_err(|e| e.to_string())?;
    let duration = start.elapsed();

    if !output.status.success() {
        return Err(format!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(duration)
}

/// Clear the cache directory
fn clear_cache(project_path: &PathBuf) {
    let cache_dir = project_path.join(".luanext-cache");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).ok();
    }
}

/// Benchmark: Cache hit vs cache miss performance
fn benchmark_cache_hit_vs_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_vs_miss");
    group.sample_size(10);

    let project = generate_cache_test_project(50, 10);
    let project_path = project.path().to_path_buf();

    // First compile to populate cache
    compile_with_cache(&project_path, true).expect("Initial compilation failed");

    // Benchmark cache hit
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            compile_with_cache(&project_path, true).expect("Cached compilation failed");
        })
    });

    // Benchmark cache miss
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            clear_cache(&project_path);
            compile_with_cache(&project_path, true).expect("Uncached compilation failed");
        })
    });

    group.finish();
}

/// Benchmark: Incremental compilation after single file change
fn benchmark_incremental_after_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_compilation");
    group.sample_size(10);

    for module_count in [25, 50, 100] {
        let project = generate_cache_test_project(module_count, 10);
        let project_path = project.path().to_path_buf();

        // Initial compilation to populate cache
        compile_with_cache(&project_path, true).expect("Initial compilation failed");

        let module_to_edit = project_path.join("module_25.luax");
        let original_content = fs::read_to_string(&module_to_edit).expect("Failed to read module");

        group.bench_with_input(
            BenchmarkId::new("file_change", module_count),
            &(project_path.clone(), module_to_edit.clone(), original_content.clone()),
            |b, (path, module_path, orig_content)| {
                b.iter(|| {
                    // Modify the file (add a comment)
                    let modified = orig_content.clone() + "\n-- Modified\n";
                    fs::write(module_path, &modified).expect("Failed to write");

                    // Compile incrementally
                    compile_with_cache(path, true).expect("Incremental compilation failed");

                    // Restore for next iteration
                    fs::write(module_path, orig_content).expect("Failed to restore");
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Cache effectiveness with different project sizes
fn benchmark_cache_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scaling");
    group.sample_size(10);

    for module_count in [10, 25, 50, 100] {
        let project = generate_cache_test_project(module_count, 10);
        let project_path = project.path().to_path_buf();

        // Clear cache before benchmark
        clear_cache(&project_path);

        // First compilation (populate cache)
        compile_with_cache(&project_path, true).expect("Initial compilation failed");

        // Second compilation (from cache)
        group.bench_with_input(
            BenchmarkId::new("modules", module_count),
            &project_path,
            |b, path| {
                b.iter(|| {
                    compile_with_cache(path, true).expect("Cached compilation failed");
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Cross-file dependency invalidation
fn benchmark_dependency_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_invalidation");
    group.sample_size(10);

    let project = generate_cache_test_project(50, 10);
    let project_path = project.path().to_path_buf();

    // Initial compilation
    compile_with_cache(&project_path, true).expect("Initial compilation failed");

    // Edit a "root" module that others depend on
    let module_0 = project_path.join("module_0.luax");
    let original_content = fs::read_to_string(&module_0).expect("Failed to read");

    group.bench_function("edit_root_dependency", |b| {
        b.iter(|| {
            // Modify module_0 (others import from it)
            let modified = original_content.clone() + "\n-- Root change\n";
            fs::write(&module_0, &modified).expect("Failed to write");

            // Recompile (should invalidate dependent modules)
            compile_with_cache(&project_path, true).expect("Compilation failed");

            // Restore
            fs::write(&module_0, &original_content).expect("Failed to restore");
        })
    });

    // Edit a "leaf" module that nothing depends on
    let module_49 = project_path.join("module_49.luax");
    let leaf_content = fs::read_to_string(&module_49).expect("Failed to read");

    group.bench_function("edit_leaf_module", |b| {
        b.iter(|| {
            // Modify module_49 (nothing imports from it)
            let modified = leaf_content.clone() + "\n-- Leaf change\n";
            fs::write(&module_49, &modified).expect("Failed to write");

            // Recompile (should only recompile this module)
            compile_with_cache(&project_path, true).expect("Compilation failed");

            // Restore
            fs::write(&module_49, &leaf_content).expect("Failed to restore");
        })
    });

    group.finish();
}

/// Benchmark: Re-export chain caching
fn benchmark_reexport_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("reexport_caching");
    group.sample_size(10);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create base modules
    for i in 0..5 {
        let content = generate_cached_module(&format!("base_{}", i), 5, 3);
        let path = temp_dir.path().join(format!("base_{}.luax", i));
        fs::write(path, content).expect("Failed to write");
    }

    // Create re-export chain
    for depth in 1..=5 {
        let mut reexport = format!("-- Re-export level {}\n\n", depth);
        if depth == 1 {
            // First level re-exports from base
            for i in 0..5 {
                reexport.push_str(&format!("export type * from \"./base_{}\"\n", i));
            }
        } else {
            // Deeper levels re-export from previous level
            reexport.push_str(&format!("export type * from \"./reexport_{}\"\n", depth - 1));
        }

        let path = temp_dir.path().join(format!("reexport_{}.luax", depth));
        fs::write(path, reexport).expect("Failed to write");
    }

    // Create main that imports from deepest re-export
    let main_content = r#"
import type { ICache0 } from "./reexport_5"

function main(): void
    print("Re-export test")
end
"#;
    let main_path = temp_dir.path().join("main.luax");
    fs::write(main_path, main_content).expect("Failed to write main");

    let project_path = temp_dir.path().to_path_buf();

    // Initial compilation
    clear_cache(&project_path);
    compile_with_cache(&project_path, true).expect("Initial compilation failed");

    group.bench_function("deep_reexport_cached", |b| {
        b.iter(|| {
            compile_with_cache(&project_path, true).expect("Compilation failed");
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_hit_vs_miss,
    benchmark_incremental_after_edit,
    benchmark_cache_scaling,
    benchmark_dependency_invalidation,
    benchmark_reexport_caching
);
criterion_main!(benches);
