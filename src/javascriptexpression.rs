use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display};

use super::expression::{CompiledExpression, Parser};
use crate::{
    expression::ParseError,
    interpreter::data::{CompilationVarContext, ScenarioRuntimeError, VarContext, VarValue},
};
use js_sandbox::{AnyError, Script};
use serde_json::Value;

/*
I use global state to cache compiled scripts. This is not what I'd like to do, but:
- Script.call mutates self.
- I don't want to make all the expression/scenario API mutable - it shouldn't be this way.
- I couldn't find any way to invoke Script (or any other library to invoke JS expressions) without mutating it. This is reasonable,
  because in general JS expressions can mutate global state of JS Runtime... Hopefuly in the future it will be possible with FrozenRealms or sth
  like that. Currently, the implementation is "unsafe" - one write `Object.global = input` and pass the state to next invocation on a given thread...

Other way to solve this is to parse Script on each invocation, but it's hopelessly inefficient then.
*/
thread_local! {
    static CACHED_SCRIPTS: RefCell<HashMap<String,Script>>  = RefCell::new(HashMap::from([]));
}

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
        let _compiled = Script::from_string(&expanded).map_err(|err| {
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

impl JavascriptExpression {
    fn execute_script(
        script: &mut Script,
        input_data: &VarContext,
    ) -> Result<VarValue, ScenarioRuntimeError> {
        let converted = serde_json::to_value(&input_data.to_external_form())
            .map_err(JavascriptExecutionError::InputParse)?;
        script
            .call::<(Value,), Value>("run", (converted,))
            .map_err(|err| ScenarioRuntimeError::from(JavascriptExecutionError::RuntimeError(err)))
    }
}

impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: &VarContext) -> Result<VarValue, ScenarioRuntimeError> {
        CACHED_SCRIPTS.with(|c| {
            let mut map = c.borrow_mut();
            if !map.contains_key(&self.transformed) {
                let expression = Script::from_string(&self.transformed)
                    .map_err(JavascriptExecutionError::ScriptParse)?;
                map.insert(self.transformed.clone(), expression);
            }
            //we are sure the key is present
            let expression = map.get_mut(&self.transformed).unwrap();
            JavascriptExpression::execute_script(expression, input_data)
        })
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
