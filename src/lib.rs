pub mod data;
pub mod runtime;
pub mod scenario;
pub mod expression;

use runtime::data::{InputData, OutputData, ScenarioError};
use serde_json::Value;
use std::collections::HashMap;

pub fn interpret_scenario(file_name: &str, input_str: &str) -> Result<OutputData, ScenarioError> {

    fn map_error(_error: std::io::Error) -> ScenarioError {
        return ScenarioError(String::from("Failed to read"))
    }
    fn map_error_json(_error: serde_json::Error) -> ScenarioError {
        return ScenarioError(String::from("Failed to read"))
    }

    let scenario = data::parse::parse(&file_name).map_err(map_error)?;
    let runtime = runtime::compiler::compile(&scenario)?;

    let input: Value = serde_json::from_str(input_str).map_err(map_error_json)?;
    let input_data: HashMap<String, Value> = HashMap::from([(String::from("input"), input)]);

    return runtime.run(&mut InputData(input_data));
    
}