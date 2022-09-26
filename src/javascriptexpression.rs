use std::{error::Error, fmt::Display};

use super::expression::{CompiledExpression, Parser};
use crate::{
    expression::ParseError,
    interpreter::data::{CompilationVarContext, ScenarioRuntimeError, VarContext, VarValue},
};
use js_sandbox::{AnyError, Script};
use serde_json::Value;

pub struct JavaScriptParser;

impl Parser for JavaScriptParser {
    fn parse(
        &self,
        expression: &str,
        var_context: &CompilationVarContext,
    ) -> Result<Box<dyn CompiledExpression>, Box<dyn ParseError>> {
        let keys = var_context
            .0
            .keys()
            //TODO: is it possible to get rid of it?
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        let expanded = format!(
            r#"function run (argMap) {{
            const {{ {} }} = argMap
            return ({})
        }}"#,
            keys, expression
        );
        let expr = JavascriptExpression {
            transformed: expanded,
        };
        Ok(Box::new(expr))
    }
}

struct JavascriptExpression {
    transformed: String,
}

impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: &VarContext) -> Result<VarValue, ScenarioRuntimeError> {
        let mut expression = Script::from_string(&self.transformed)
            .map_err(JavascriptExecutionError::ScriptParse)?;
        let converted = serde_json::to_value(&input_data.to_external_form())
            .map_err(JavascriptExecutionError::InputParse)?;
        expression
            .call::<Value, Value>("run", &converted)
            .map_err(|err| ScenarioRuntimeError::from(JavascriptExecutionError::RuntimeError(err)))
    }
}

#[derive(Debug)]
enum JavascriptExecutionError {
    ScriptParse(AnyError),
    InputParse(serde_json::Error),
    RuntimeError(AnyError),
}

impl From<JavascriptExecutionError> for ScenarioRuntimeError {
    fn from(error: JavascriptExecutionError) -> Self {
        ScenarioRuntimeError::ExpressionError(Box::new(error))
    }
}

impl Display for JavascriptExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //this should be nicely handled, just like in ForEachError...
        write!(f, "TODO...")
    }
}
impl Error for JavascriptExecutionError {}

#[cfg(test)]
mod tests {
    use crate::{
        expression::Parser,
        interpreter::data::{CompilationVarContext, VarContext},
        javascriptexpression::JavaScriptParser,
    };
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_simple_expression() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser
            .parse("10 + 5", &CompilationVarContext(HashMap::new()))
            .unwrap();
        let res = expr.execute(&VarContext::empty())?;
        assert_eq!(res, json!(15));
        Ok(())
    }

    #[test]
    fn test_expression_with_variable() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser
            .parse("input + 5", &CompilationVarContext::default())
            .unwrap();
        let res = expr.execute(&VarContext::default_input(json!(10)))?;
        assert_eq!(res, serde_json::to_value(15)?);
        Ok(())
    }
}
