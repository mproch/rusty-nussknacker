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

pub(super) fn compile(ctx: CompilationContext, nexts: &[Case]) -> CompilationResult {
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
        let mut result = Ok(ScenarioOutput(vec![]));
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        interpreter::data::{VarContext, DEFAULT_INPUT_NAME},
        scenariomodel::{Case, Node, NodeId},
    };

    use super::super::tests;

    #[test]
    fn test_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let left_sink_id = NodeId::new("sink1");
        let right_sink_id = NodeId::new("sink2");

        let left_case = Case {
            expression: tests::js("input > 0"),
            nodes: tests::sink(&left_sink_id),
        };
        let right_case = Case {
            expression: tests::js("input <= 0"),
            nodes: tests::sink(&right_sink_id),
        };

        let compiled = tests::compile_node(
            Node::Switch {
                id: NodeId::new("node_id"),
                nexts: vec![left_case, right_case],
            },
            &[],
        )?;

        let input = json!(8);
        let result = compiled.run(&VarContext::default_context_for_value(input.clone()))?;
        assert_eq!(
            result.var_in_sink(&left_sink_id, DEFAULT_INPUT_NAME),
            [Some(&input)]
        );
        assert_eq!(result.var_in_sink(&right_sink_id, DEFAULT_INPUT_NAME), []);

        let input = json!(-5);
        let result = compiled.run(&VarContext::default_context_for_value(input.clone()))?;
        assert_eq!(result.var_in_sink(&left_sink_id, DEFAULT_INPUT_NAME), []);
        assert_eq!(
            result.var_in_sink(&right_sink_id, DEFAULT_INPUT_NAME),
            [Some(&input)]
        );

        Ok(())
    }
}
