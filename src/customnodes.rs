use std::collections::HashMap;
use crate::runtime::{CustomNodeImpl, Interpreter, data::{OutputData, ScenarioRuntimeError, VarValue, InputData}};
use serde_json::Value::{Array, self};
pub struct ForEach;

impl CustomNodeImpl for ForEach {
    fn run(&self, output_var: &str, parameters: HashMap<String, VarValue>, data: &InputData, next_part: &dyn Interpreter) -> Result<OutputData, ScenarioRuntimeError> {
        let run = |v: &Value| {
            data.insert(output_var, v.clone());
            next_part.run(data)  
        };
        match parameters.get("value") {
            Some(Array(values)) => {
                let output_result: Result<Vec<OutputData>, ScenarioRuntimeError> = values.iter().map(run).collect();
                output_result.map(OutputData::flatten)
            },
            Some(other) => Err(ScenarioRuntimeError(format!("Unexpected value: {}", other))),
            None => Err(ScenarioRuntimeError(String::from("Fail")))
        }
    }
}