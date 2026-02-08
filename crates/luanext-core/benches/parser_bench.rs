use bumpalo::Bump;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use typedlua_core::diagnostics::CollectingDiagnosticHandler;
use typedlua_parser::lexer::Lexer;
use typedlua_parser::parser::Parser;
use typedlua_parser::string_interner::StringInterner;

fn bench_parser_simple(c: &mut Criterion) {
    let source = r#"
        const x: number = 42
        const y: string = "hello"
        function add(a: number, b: number): number {
            return a + b
        }
    "#;

    c.bench_function("parser_simple", |b| {
        b.iter(|| {
            let handler = Arc::new(CollectingDiagnosticHandler::new());
            let (interner, common_ids) = StringInterner::new_with_common_identifiers();
            let arena = Bump::new();
            let mut lexer = Lexer::new(black_box(source), handler.clone(), &interner);
            if let Ok(tokens) = lexer.tokenize() {
                let mut parser = Parser::new(tokens, handler, &interner, &common_ids, &arena);
                if let Ok(program) = parser.parse() {
                    black_box(&program);
                }
            }
        })
    });
}

fn bench_parser_class(c: &mut Criterion) {
    let source = r#"
        class User {
            public name: string
            private age: number

            constructor(name: string, age: number) {
                this.name = name
                this.age = age
            }

            public greet(): void {
                print(`Hello, ${this.name}!`)
            }
        }
    "#;

    c.bench_function("parser_class", |b| {
        b.iter(|| {
            let handler = Arc::new(CollectingDiagnosticHandler::new());
            let (interner, common_ids) = StringInterner::new_with_common_identifiers();
            let arena = Bump::new();
            let mut lexer = Lexer::new(black_box(source), handler.clone(), &interner);
            if let Ok(tokens) = lexer.tokenize() {
                let mut parser = Parser::new(tokens, handler, &interner, &common_ids, &arena);
                if let Ok(program) = parser.parse() {
                    black_box(&program);
                }
            }
        })
    });
}

fn bench_parser_interface(c: &mut Criterion) {
    let source = r#"
        interface Printable {
            print(): void
        }

        interface Serializable {
            serialize(): string
            deserialize(data: string): void
        }

        class Document implements Printable, Serializable {
            private content: string

            constructor(content: string) {
                this.content = content
            }

            public print(): void {
                print(this.content)
            }

            public serialize(): string {
                return this.content
            }

            public deserialize(data: string): void {
                this.content = data
            }
        }
    "#;

    c.bench_function("parser_interface", |b| {
        b.iter(|| {
            let handler = Arc::new(CollectingDiagnosticHandler::new());
            let (interner, common_ids) = StringInterner::new_with_common_identifiers();
            let arena = Bump::new();
            let mut lexer = Lexer::new(black_box(source), handler.clone(), &interner);
            if let Ok(tokens) = lexer.tokenize() {
                let mut parser = Parser::new(tokens, handler, &interner, &common_ids, &arena);
                if let Ok(program) = parser.parse() {
                    black_box(&program);
                }
            }
        })
    });
}

fn bench_parser_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_scaling");

    for size in [10, 50, 100].iter() {
        let source = (0..*size)
            .map(|i| {
                format!(
                    "function func{}(x: number): number {{ return x + {} }}",
                    i, i
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        group.bench_with_input(BenchmarkId::from_parameter(size), &source, |b, s| {
            b.iter(|| {
                let handler = Arc::new(CollectingDiagnosticHandler::new());
                let (interner, common_ids) = StringInterner::new_with_common_identifiers();
                let arena = Bump::new();
                let mut lexer = Lexer::new(black_box(s), handler.clone(), &interner);
                if let Ok(tokens) = lexer.tokenize() {
                    let mut parser = Parser::new(tokens, handler, &interner, &common_ids, &arena);
                    if let Ok(program) = parser.parse() {
                        black_box(&program);
                    }
                }
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parser_simple,
    bench_parser_class,
    bench_parser_interface,
    bench_parser_size_scaling
);
criterion_main!(benches);
