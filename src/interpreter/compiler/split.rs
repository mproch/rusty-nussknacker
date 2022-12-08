use async_trait::async_trait;
use futures::future::join_all;

use crate::{
    interpreter::{
        data::{ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext},
        CompilationResult, Interpreter,
    },
    scenariomodel::Node,
};

use super::CompilationContext;

pub(super) struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>,
}

pub(super) fn compile(ctx: CompilationContext, nexts: &[Vec<Node>]) -> CompilationResult {
    let compiled: Result<Vec<Box<dyn Interpreter>>, ScenarioCompilationError> = nexts
        .iter()
        .map(|n| (ctx.compiler)(&n[..], ctx.var_names))
        .collect();
    ctx.assert_end(Box::new(CompiledSplit { nexts: compiled? }))
}

#[async_trait]
impl Interpreter for CompiledSplit {
    async fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
            join_all(self.nexts.iter().map(|one| one.run(data)))
                .await
                .into_iter()
                .collect();
        output_result.map(ScenarioOutput::flatten)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::data::{VarContext, DEFAULT_INPUT_NAME},
        scenariomodel::{Node, NodeId},
    };
    use serde_json::json;
    use tokio_test::block_on;

    use super::super::tests;

    #[test]
    fn test_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let branch1 = NodeId::new("branch1");
        let branch2 = NodeId::new("branch2");

        let node_to_test = Node::Split {
            id: NodeId::new("split"),
            nexts: vec![tests::sink(&branch1), tests::sink(&branch2)],
        };

        let compiled = tests::compile_node(node_to_test, &[])?;

        let input = json!("to_copy");
        let result = block_on(compiled.run(&VarContext::default_context_for_value(input.clone())))?;
        assert_eq!(
            result.var_in_sink(&branch1, DEFAULT_INPUT_NAME),
            [Some(&input)]
        );
        assert_eq!(
            result.var_in_sink(&branch2, DEFAULT_INPUT_NAME),
            [Some(&input)]
        );

        Ok(())
    }
}
