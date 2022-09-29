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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{interpreter::data::VarContext, scenariomodel::{Node, NodeId, Case}};

    use super::super::tests;

    #[test]
    fn test_outputs() -> Result<(), Box<dyn std::error::Error>> {
        
        let rest1 = vec![Node::Sink { id: NodeId::new("sink1") }];
        let case1 = Case { expression: tests::js("input > 0"), nodes: rest1.clone() };
        let rest2 = vec![Node::Sink { id: NodeId::new("sink1") }];
        let case2 = Case { expression: tests::js("input <= 0"), nodes: rest2.clone() };
        let cases = &[case1.clone(), case2.clone()];

        let compiled =
            tests::with_stub_context(&|ctx| super::compile(ctx, cases), &rest1.clone())?;
        let input = json!(8);
        let result = compiled.run(&VarContext::default_input(input.clone()))?;
        assert_eq!(
            result.var_in_sink(tests::output_node_id(), "input"),
            [Some(&input)]
        );

        let compiled =
            tests::with_stub_context(&|ctx| super::compile(ctx, cases), &rest2.clone())?;
        let input = json!(-4);
        let result = compiled.run(&VarContext::default_input(input.clone()))?;
        assert_eq!(
            result.var_in_sink(tests::output_node_id(), "input"),
            [Some(&input)]
        );


        Ok(())
    }
}
