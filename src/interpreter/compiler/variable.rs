use crate::{
    expression::CompiledExpression,
    interpreter::{
        data::{ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext},
        Interpreter,
    },
    scenariomodel::Expression,
};

use super::CompilationContext;

struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String,
}

pub fn compile(
    ctx: CompilationContext,
    var_name: &str,
    raw_expression: &Expression,
) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
    let expression = ctx
        .parser
        .parse(ctx.node_id, raw_expression, ctx.var_names)?;
    let rest = (ctx.compiler)(ctx.rest, &ctx.var_names.with_var(ctx.node_id, var_name)?)?;
    Ok(Box::new(CompiledVariable {
        rest,
        expression,
        var_name: String::from(var_name),
    }))
}

impl Interpreter for CompiledVariable {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        let with_var = data.insert(&self.var_name, result);
        self.rest.run(&with_var)
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::{compiler::tests, data::VarContext};
    use serde_json::json;

    #[test]
    fn test_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let expression = tests::js("input + '-suffix'");
        let output_name = "test_output";
        let compiled =
            tests::with_stub_context(&|ctx| super::compile(ctx, output_name, &expression))?;

        let input = json!("terefere");
        let output = json!("terefere-suffix");

        let result = compiled.run(&VarContext::default_input(input))?;
        assert_eq!(
            result.var_in_sink(tests::output_node_id(), output_name),
            [Some(&output)]
        );

        Ok(())
    }
}
