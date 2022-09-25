use std::collections::HashMap;

use self::data::{ScenarioRuntimeError, VarValue, ScenarioOutput, VarContext};

pub mod data;
pub mod compiler;

pub trait CustomNodeImpl {
    fn run(&self, output_var: &str, parameters: HashMap<String, VarValue>, input: &VarContext, next_part: &dyn Interpreter) -> Result<ScenarioOutput, ScenarioRuntimeError>;
}

pub trait Interpreter {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError>;
}