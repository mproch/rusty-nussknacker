use std::{error::Error, fmt::Display};

use super::expression::{CompiledExpression, Parser};
use crate::{
    expression::ParseError,
    interpreter::data::{CompilationVarContext, ScenarioRuntimeError, VarContext, VarValue},
};
use js_sandbox::{AnyError, Script};
use serde_json::Value;

pub struct JavaScriptParser;

///Essentially we wrap given expression in a function, that takes one argument (as it's required by js-sandbox crate), and destructures in
///as we assume that the argument is an object composed of all variables available
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
        //we ignore the result, as we just want to check if expression is correct
        let _try_to_compile = Script::from_string(&expanded).map_err(|err| {
            //looks clumsy, but type inference fails here :/
            let ret: Box<dyn ParseError> = Box::new(JavascriptParseError(err));
            ret
        })?;
        Ok(Box::new(JavascriptExpression {
            transformed: expanded,
        }))
    }
}

struct JavascriptExpression {
    transformed: String,
}

//This is inefficient implementation, as it parses expression on each invocation. The reason is that
//call mutates the script. Don't want to figure out how to handle it at the moment.
//(In JVM in similar cases we had compiled expressions stored in ThreadLocal to deal with concurrency issues, but
//here the problem is - should CompiledExpression::execute be allowed to mutate the expression??)
impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: &VarContext) -> Result<VarValue, ScenarioRuntimeError> {
        let mut expression = Script::from_string(&self.transformed)
            .map_err(JavascriptExecutionError::ScriptParse)?;
        let converted = serde_json::to_value(&input_data.to_external_form())
            .map_err(JavascriptExecutionError::InputParse)?;
        expression
            .call::<(Value,), Value>("run", (converted,))
            .map_err(|err| ScenarioRuntimeError::from(JavascriptExecutionError::RuntimeError(err)))
    }
}

#[derive(Debug)]
struct JavascriptParseError(AnyError);

impl Error for JavascriptParseError {}

impl ParseError for JavascriptParseError {}

impl Display for JavascriptParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //this should be nicely handled, just like in ForEachError...
        write!(f, "Cannot parse: {}", self.0)
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
        write!(f, "Error occurred: {:?}", self)
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

    #[test]
    fn test_simple_expression() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser
            .parse("10 + 5", &CompilationVarContext::default())
            .unwrap();
        let res = expr.execute(&VarContext::empty())?;
        assert_eq!(res, json!(15));
        Ok(())
    }

    #[test]
    fn test_parse_wrong_expression() {
        let expr = JavaScriptParser.parse("return aaaa", &CompilationVarContext::default());
        assert!(expr.is_err());
    }

    #[test]
    fn test_expression_with_variable() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser
            .parse("input + 5", &CompilationVarContext::default())
            .unwrap();
        let res = expr.execute(&VarContext::default_input(json!(10)))?;
        assert_eq!(res, json!(15));
        Ok(())
    }

    #[test]
    fn test_nested_multiline_expression() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser
            .parse(
                "[input].map(x => {
               function add(v1, v2) { return v1 + v2; }
               return add(x, '+suffix');
            })[0]",
                &CompilationVarContext::default(),
            )
            .unwrap();
        let res = expr.execute(&VarContext::default_input(json!("my_input")))?;
        assert_eq!(res, json!("my_input+suffix"));
        Ok(())
    }
}
