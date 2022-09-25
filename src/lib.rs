pub mod data;
pub mod runtime;
pub mod scenario;
pub mod expression;
pub mod customnodes;

use runtime::{data::{InputData, OutputData, ScenarioCompilationError, ScenarioRuntimeError}, Interpreter};
use serde_json::Value;

use crate::runtime::compiler::Compiler;

pub fn create_interpreter(file_name: &str) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {

    fn map_error(_error: std::io::Error) -> ScenarioCompilationError {
        ScenarioCompilationError(String::from("Failed to read"))
    }
    let scenario = data::parse::parse(file_name).map_err(map_error)?;
    let compiler: Compiler = Default::default();
    compiler.compile(&scenario)
}

pub fn invoke_interpreter(runtime: &dyn Interpreter, input: &str) -> Result<OutputData, ScenarioRuntimeError> {
    fn map_error_json(_error: serde_json::Error) -> ScenarioRuntimeError {
        ScenarioRuntimeError(String::from("Failed to read"))
    }

    let input: Value = serde_json::from_str(input).map_err(map_error_json)?;
    runtime.run(&InputData::default_input(input))
}
