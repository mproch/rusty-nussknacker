
use crate::{runtime::data::{ScenarioCompilationError, ScenarioRuntimeError, VarContext, InputData, VarValue}};
use super::{CompiledExpression, Parser};
use js_sandbox::Script;
use serde_json::Value;

pub struct JavaScriptParser;

impl Parser for JavaScriptParser {
    fn parse(&self, expression: &str, var_context: &VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioCompilationError> {
        let keys = var_context.0.keys().cloned().collect::<Vec<String>>().join(", ");
        let expanded = format!(r#"function run (argMap) {{
            const {{ {} }} = argMap
            return ({})
        }}"#, keys, expression);
        let expr = JavascriptExpression { transformed: expanded };
        Ok(Box::new(expr))           
    }
}

struct JavascriptExpression {
    transformed: String
}

impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: &InputData) -> Result<VarValue, ScenarioRuntimeError> {
        let mut expression = Script::from_string(&self.transformed).map_err(|err| ScenarioRuntimeError(err.to_string()))?;
        let converted = serde_json::to_value(&input_data.to_serialize()).map_err(|err| ScenarioRuntimeError(err.to_string()))?;
        expression.call::<Value, Value>("run", &converted).map_err(|err| ScenarioRuntimeError(err.to_string()))   
    }
}


#[test]
fn test_simple_expression() {
    use std::collections::HashMap;
    
    let expr = JavaScriptParser.parse("10 + 5", &VarContext(HashMap::new())).unwrap();
    let res = expr.execute(&InputData(HashMap::from([])));
    assert_eq!(res.unwrap(), serde_json::to_value(15).unwrap());
}

#[test]
fn test_expression_with_variable() {
    use std::collections::HashMap;
    use std::rc::Rc;

    let expr = JavaScriptParser.parse("ala + 5", &VarContext(HashMap::from([(String::from("ala"), ())]))).unwrap();
    let res = expr.execute(&InputData(HashMap::from([(String::from("ala"), Rc::new(serde_json::to_value(10).unwrap()))])));
    assert_eq!(res.unwrap(), serde_json::to_value(15).unwrap());
}
