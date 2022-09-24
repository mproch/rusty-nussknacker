pub mod data;
pub mod runtime;
pub mod scenario;
pub mod expression;
pub mod customnodes;

use runtime::data::{InputData, OutputData, ScenarioError, ScenarioError::*};
use serde_json::Value;

use crate::runtime::compiler::Compiler;

pub fn interpret_scenario(file_name: &str, input_str: &str) -> Result<OutputData, ScenarioError> {

    fn map_error(_error: std::io::Error) -> ScenarioError {
        ScenarioRuntimeError(String::from("Failed to read"))
    }
    fn map_error_json(_error: serde_json::Error) -> ScenarioError {
        ScenarioRuntimeError(String::from("Failed to read"))
    }

    let scenario = data::parse::parse(file_name).map_err(map_error)?;
    let compiler: Compiler = Default::default();
    let runtime = compiler.compile(&scenario)?;

    let input: Value = serde_json::from_str(input_str).map_err(map_error_json)?;

    runtime.run(&InputData::default_input(input))
}