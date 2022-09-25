use std::collections::HashMap;

use self::data::{ScenarioRuntimeError, VarValue, OutputData, InputData};

pub mod data;
pub mod compiler;

pub trait CustomNodeImpl {
    fn run(&self, output_var: &str, parameters: HashMap<String, VarValue>, input: &InputData, next_part: &dyn Interpreter) -> Result<OutputData, ScenarioRuntimeError>;
}

pub trait Interpreter {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioRuntimeError>;
}