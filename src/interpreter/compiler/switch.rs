use crate::{
    expression::CompiledExpression,
    interpreter::{
        data::{ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext},
        CompilationResult, Interpreter,
    },
    scenariomodel::Case,
};
use serde_json::Value::Bool;

use super::CompilationContext;
struct CompiledSwitch {
    nexts: Vec<CompiledCase>,
}

pub fn compile(ctx: CompilationContext, nexts: &[Case]) -> CompilationResult {
    let parse_case = |case: &Case| {
        let rest = (ctx.compiler)(&case.nodes[..], ctx.var_names)?;
        let expression = ctx
            .parser
            .parse(ctx.node_id, &case.expression, ctx.var_names)?;
        Ok(CompiledCase { rest, expression })
    };
    let compiled: Result<Vec<CompiledCase>, ScenarioCompilationError> =
        nexts.iter().map(parse_case).collect();
    ctx.assert_end(Box::new(CompiledSwitch { nexts: compiled? }))
}

struct CompiledCase {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
}

impl Interpreter for CompiledSwitch {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let mut result: Result<ScenarioOutput, ScenarioRuntimeError> = Ok(ScenarioOutput(vec![]));
        for case in &self.nexts {
            let next_expression = case.expression.execute(data)?;
            let matches = (match next_expression {
                Bool(value) => Ok(value),
                other => Err(ScenarioRuntimeError::InvalidSwitchType(other)),
            })?;
            if matches {
                result = case.rest.run(data);
                break;
            }
        }
        result
    }
}
