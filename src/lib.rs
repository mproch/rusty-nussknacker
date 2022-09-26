pub mod customnodes;
pub mod expression;
pub mod interpreter;
mod javascriptexpression;
pub mod scenariomodel;

use std::path::Path;

use interpreter::{
    data::{
        ScenarioCompilationError::ScenarioReadFailure, ScenarioOutput, ScenarioRuntimeError,
        VarContext,
    },
    CompilationResult, Interpreter,
};
use serde_json::Value;

use crate::interpreter::compiler::Compiler;

pub fn create_interpreter(scenario_path: &Path) -> CompilationResult {
    let scenario = scenariomodel::parse_file(scenario_path).map_err(ScenarioReadFailure)?;
    let compiler: Compiler = Default::default();
    compiler.compile(&scenario)
}

pub fn invoke_interpreter(
    runtime: &dyn Interpreter,
    input: &str,
) -> Result<ScenarioOutput, ScenarioRuntimeError> {
    let input: Value =
        serde_json::from_str(input).map_err(ScenarioRuntimeError::CannotParseInput)?;
    runtime.run(&VarContext::default_input(input))
}
