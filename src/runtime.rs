use std::collections::HashMap;

use self::{compiler::Interpreter, data::{ScenarioError, VarValue, OutputData, InputData}};

pub mod data;
pub mod compiler;

pub trait CustomNodeImpl {
    fn run(&self, output_var: &str, parameters: HashMap<String, VarValue>, input: &InputData, next_part: &dyn Interpreter) -> Result<OutputData, ScenarioError>;
}