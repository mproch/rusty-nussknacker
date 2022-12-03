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

pub(super) fn compile(ctx: CompilationContext, expression: &Expression) -> CompilationResult {
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

    use crate::{
        interpreter::data::{VarContext, DEFAULT_INPUT_NAME},
        scenariomodel::{Node, NodeId},
    };

    use super::super::tests;

    #[test]
    fn test_filter_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let node_to_test = Node::Filter {
            id: NodeId::new("filter"),
            expression: tests::js("input>5"),
        };
        let sink_id = NodeId::new("sink1");

        let compiled = tests::compile_node(node_to_test, &tests::sink(&sink_id))?;

        let result = compiled.run(&VarContext::default_context_for_value(json!(3)))?;
        assert_eq!(result.var_in_sink(&sink_id, DEFAULT_INPUT_NAME), []);

        let input = json!(8);
        let result = compiled.run(&VarContext::default_context_for_value(input.clone()))?;
        assert_eq!(
            result.var_in_sink(&sink_id, DEFAULT_INPUT_NAME),
            [Some(&input)]
        );

        Ok(())
    }
}
