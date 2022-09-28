use crate::{
    interpreter::{
        data::{ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext},
        CompilationResult, Interpreter,
    },
    scenariomodel::Node,
};

use super::CompilationContext;

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>,
}

pub fn compile(ctx: CompilationContext, nexts: &[Vec<Node>]) -> CompilationResult {
    let compiled: Result<Vec<Box<dyn Interpreter>>, ScenarioCompilationError> = nexts
        .iter()
        .map(|n| (ctx.compiler)(&n[..], ctx.var_names))
        .collect();
    ctx.assert_end(Box::new(CompiledSplit { nexts: compiled? }))
}

impl Interpreter for CompiledSplit {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
            self.nexts.iter().map(|one| one.run(data)).collect();
        output_result.map(ScenarioOutput::flatten)
    }
}
