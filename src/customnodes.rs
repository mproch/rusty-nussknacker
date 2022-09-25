use crate::interpreter::{
    data::{ScenarioOutput, ScenarioRuntimeError, VarContext, VarValue},
    CustomNodeImpl, Interpreter,
};
use serde_json::Value::{self, Array};
use std::collections::HashMap;
pub struct ForEach;

impl CustomNodeImpl for ForEach {
    fn run(
        &self,
        output_var: &str,
        parameters: HashMap<String, VarValue>,
        data: &VarContext,
        next_part: &dyn Interpreter,
    ) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let run = |v: &Value| {
            data.insert(output_var, v.clone());
            next_part.run(data)
        };
        match parameters.get("value") {
            Some(Array(values)) => {
                let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
                    values.iter().map(run).collect();
                output_result.map(ScenarioOutput::flatten)
            }
            Some(other) => Err(ScenarioRuntimeError(format!("Unexpected value: {}", other))),
            None => Err(ScenarioRuntimeError(String::from("Fail"))),
        }
    }
}
