use async_trait::async_trait;

use crate::{
    interpreter::{
        data::{ScenarioOutput, ScenarioRuntimeError, SingleScenarioOutput, VarContext},
        CompilationResult, Interpreter,
    },
    scenariomodel::NodeId,
};

use super::CompilationContext;

pub struct CompiledSink {
    node_id: NodeId,
}

pub(super) fn compile(ctx: CompilationContext, sink_id: &NodeId) -> CompilationResult {
    ctx.assert_end(Box::new(CompiledSink {
        node_id: sink_id.clone(),
    }))
}

#[async_trait]
impl Interpreter for CompiledSink {
    async fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        Ok(ScenarioOutput(vec![SingleScenarioOutput {
            node_id: self.node_id.clone(),
            variables: data.to_external_form(),
        }]))
    }
}
