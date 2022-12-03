use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_nussknacker::{
    expression::LanguageParser,
    interpreter::data::{CompilationVarContext, VarContext},
    scenariomodel::{Expression, NodeId},
};
use serde_json::json;

pub fn simple_expression_benchmark(c: &mut Criterion) {
    let expr = LanguageParser::default()
        .parse(
            &NodeId::new("bench"),
            &Expression {
                language: String::from("javascript"),
                expression: String::from("input + 5"),
            },
            &CompilationVarContext::default(),
        )
        .unwrap();

    c.bench_function("expr input + 5", |b| {
        b.iter(|| expr.execute(&VarContext::default_context_for_value(black_box(json!(10)))))
    });
}

criterion_group!(benches, simple_expression_benchmark);
criterion_main!(benches);
