use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_nussknacker::{create_interpreter, interpreter::data::VarContext};
use serde_json::json;

pub fn simple_expression_benchmark(c: &mut Criterion) {
    let interpreter = create_interpreter(scenario("with_split.json").as_path()).unwrap();
    c.bench_function("scenario split", |b| {
        b.iter(|| {
            interpreter
                .run(black_box(&VarContext::default_context_for_value(json!(4))))
                .unwrap()
        })
    });
}

criterion_group!(benches, simple_expression_benchmark);
criterion_main!(benches);

fn scenario(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/scenarios");
    d.push(name);
    d
}
