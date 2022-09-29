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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::interpreter::data::VarContext;

    use super::super::tests;

    #[test]
    fn test_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let expression = tests::js("input>5");
        let compiled =
            tests::with_stub_context_single_output(&|ctx| super::compile(ctx, &expression))?;

        let result = compiled.run(&VarContext::default_input(json!(3)))?;
        assert_eq!(result.var_in_sink(tests::output_node_id(), "input"), []);

        let input = json!(8);
        let result = compiled.run(&VarContext::default_input(input.clone()))?;
        assert_eq!(
            result.var_in_sink(tests::output_node_id(), "input"),
            [Some(&input)]
        );

        Ok(())
    }
}
