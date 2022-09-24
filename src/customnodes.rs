use std::collections::HashMap;
use crate::runtime::{CustomNodeImpl, compiler::Interpreter, data::{OutputData, ScenarioError, VarValue, InputData}};
use crate::ScenarioError::ScenarioRuntimeError;
use serde_json::Value::{Array, self};
pub struct ForEach;

impl CustomNodeImpl for ForEach {
    fn run(&self, output_var: &str, parameters: HashMap<String, VarValue>, data: &InputData, next_part: &Box<dyn Interpreter>) -> Result<OutputData, ScenarioError> {
        let run = |v: &Value| {
            data.insert(output_var, v.clone());
            return next_part.run(data);    
        };
        match parameters.get("value") {
            Some(Array(values)) => {
                let output_result: Result<Vec<OutputData>, ScenarioError> = values.iter().map(run).collect();
                return output_result.map(OutputData::flatten);
            },
            Some(other) => Err(ScenarioRuntimeError(format!("Unexpected value: {}", other))),
            None => return Err(ScenarioRuntimeError(String::from("Fail")))
        }
    }
}