use std::{collections::HashMap, sync::Arc};

use crate::{
    expression::CompiledExpression,
    interpreter::{
        data::{
            ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext, VarValue,
        },
        CompilationResult, CustomNode, Interpreter,
    },
    scenariomodel::Parameter,
};

use super::CompilationContext;

struct CompiledCustomNode {
    rest: Box<dyn Interpreter>,
    output_var: String,
    params: HashMap<String, Box<dyn CompiledExpression>>,
    custom_node: Arc<dyn CustomNode>,
}

pub(super) fn compile(
    ctx: CompilationContext,
    output_var: &str,
    parameters: &[Parameter],
    implementation: &Arc<dyn CustomNode>,
) -> CompilationResult {
    let next_part = (ctx.compiler)(ctx.rest, &ctx.var_names.with_var(ctx.node_id, output_var)?)?;
    let compiled_parameters: Result<
        HashMap<String, Box<dyn CompiledExpression>>,
        ScenarioCompilationError,
    > = parameters
        .iter()
        .map(|p| compile_parameter(&ctx, p))
        .collect();
    Ok(Box::new(CompiledCustomNode {
        rest: next_part,
        output_var: String::from(output_var),
        params: compiled_parameters?,
        custom_node: implementation.clone(),
    }))
}

fn compile_parameter(
    ctx: &CompilationContext,
    parameter: &Parameter,
) -> Result<(String, Box<dyn CompiledExpression>), ScenarioCompilationError> {
    let compiled_expression =
        ctx.parser
            .parse(ctx.node_id, &parameter.expression, ctx.var_names)?;
    Ok((parameter.name.clone(), compiled_expression))
}

impl Interpreter for CompiledCustomNode {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let parameters: Result<HashMap<String, VarValue>, ScenarioRuntimeError> = self
            .params
            .iter()
            //I was hoping for some nice variant of mapValues...
            .map(|e| e.1.execute(data).map(|r| (String::from(e.0), r)))
            .collect();
        self.custom_node
            .run(&self.output_var, &parameters?, data, self.rest.as_ref())
    }
}
