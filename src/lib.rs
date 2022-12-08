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

use crate::interpreter::compiler::Compiler;

pub fn create_interpreter(scenario_path: &Path) -> CompilationResult {
    let scenario = scenariomodel::parse_file(scenario_path).map_err(ScenarioReadFailure)?;
    let compiler = Compiler::default();
    compiler.compile(&scenario)
}

pub async fn invoke_interpreter(
    runtime: &dyn Interpreter,
    input: &str,
) -> Result<ScenarioOutput, ScenarioRuntimeError> {
    let input = serde_json::from_str(input).map_err(ScenarioRuntimeError::CannotParseInput)?;
    runtime
        .run(&VarContext::default_context_for_value(input))
        .await
}
