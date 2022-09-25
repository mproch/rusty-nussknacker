pub mod scenariomodel;
pub mod runtime;
pub mod expression;
pub mod customnodes;
mod javascriptexpression;

use runtime::{data::{VarContext, ScenarioOutput, ScenarioCompilationError, ScenarioRuntimeError}, Interpreter};
use serde_json::Value;

use crate::runtime::compiler::Compiler;

pub fn create_interpreter(file_name: &str) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {

    fn map_error(_error: std::io::Error) -> ScenarioCompilationError {
        ScenarioCompilationError(String::from("Failed to read"))
    }
    let scenario = scenariomodel::parse_file(file_name).map_err(map_error)?;
    let compiler: Compiler = Default::default();
    compiler.compile(&scenario)
}

pub fn invoke_interpreter(runtime: &dyn Interpreter, input: &str) -> Result<ScenarioOutput, ScenarioRuntimeError> {
    fn map_error_json(_error: serde_json::Error) -> ScenarioRuntimeError {
        ScenarioRuntimeError(String::from("Failed to read"))
    }

    let input: Value = serde_json::from_str(input).map_err(map_error_json)?;
    runtime.run(&VarContext::default_input(input))
}
