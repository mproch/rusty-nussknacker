use super::CompilationContext;
use crate::{
    expression::CompiledExpression,
    interpreter::{
        data::{ScenarioOutput, ScenarioRuntimeError, VarContext},
        CompilationResult, Interpreter,
    },
    scenariomodel::Expression,
};
use serde_json::Value::Bool;

struct CompiledFilter {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
}

pub fn compile(ctx: CompilationContext, expression: &Expression) -> CompilationResult {
    let rest = (ctx.compiler)(ctx.rest, ctx.var_names)?;
    let expression = ctx.parser.parse(ctx.node_id, expression, ctx.var_names)?;
    let res = CompiledFilter { rest, expression };
    Ok(Box::new(res))
}

impl Interpreter for CompiledFilter {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        match result {
            Bool(true) => self.rest.run(data),
            Bool(false) => Ok(ScenarioOutput(vec![])),
            other => Err(ScenarioRuntimeError::InvalidSwitchType(other)),
        }
    }
}
