
use core::panic;
use std::collections::HashMap;

use crate::{runtime::data::{ScenarioError, VarContext, InputData, VarValue}, data::jsonmodel::Expression};
use super::CompiledExpression;
use js_sandbox::Script;
use serde_json::Value;

pub fn parse(expression: Expression, var_context: VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioError> {
    match expression.language.as_str() {
        "javascript" => parse_javascript(&expression.expression, var_context),
        _ => panic!("Unknown language")
    }
}

fn parse_javascript(expression: &str, var_context: VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioError>{
    let keys = var_context.0.keys().cloned().collect::<Vec<String>>().join(", ");
    let expanded = format!(r#"function run (argMap) {{
        const {{ {} }} = argMap
        return ({})
    }}"#, keys, expression);
    let expr = JavascriptExpression { transformed: expanded };
    return Ok(Box::new(expr));
}

struct JavascriptExpression {
    transformed: String
}

impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: InputData) -> Result<VarValue, ScenarioError> {
        let mut expression = Script::from_string(&self.transformed).map_err(|err| ScenarioError(err.to_string()))?;
        let converted = serde_json::to_value(input_data.0).map_err(|err| ScenarioError(err.to_string()))?;
        return expression.call::<Value, Value>("run", &converted).map_err(|err| ScenarioError(err.to_string()));    
    }
}

#[test]
fn test_simple_expression() -> Result<(), ScenarioError> {
    let expr = parse(Expression { language: String::from("javascript"), expression: String::from("10 + 5") }, VarContext(HashMap::new()))?;
    expr.execute(InputData(HashMap::new()))?;
    return Ok(());
}

#[test]
fn test_expression_with_variable() -> Result<(), ScenarioError> {
    let expr = parse(Expression { language: String::from("javascript"), expression: String::from("ala + 5") }, VarContext(HashMap::from([(String::from("ala"), ())])))?;
    let res = expr.execute(InputData(HashMap::from([(String::from("ala"), serde_json::to_value(10).unwrap())])));
    assert_eq!(res.unwrap(), serde_json::to_value(15).unwrap());
    return Ok(());    
}