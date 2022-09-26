use crate::interpreter::{
    data::{ScenarioOutput, ScenarioRuntimeError, VarContext, VarValue},
    CustomNodeImpl, Interpreter,
};
use serde_json::Value::{self, Array};
use std::{collections::HashMap, error::Error, fmt::Display};
pub struct ForEach;

const VALUE_PARAM: &str = "value";

///The components requires "value" parameter of array type.
/// For each element of array, the subsequent part of the scenario is invoked, with the element passed as an output variable
impl CustomNodeImpl for ForEach {
    fn run(
        &self,
        output_var: &str,
        parameters: HashMap<String, VarValue>,
        data: &VarContext,
        next_part: &dyn Interpreter,
    ) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let run = |v: &Value| {
            let new_data = data.insert(output_var, v.clone());
            next_part.run(&new_data)
        };
        match parameters.get(VALUE_PARAM) {
            Some(Array(values)) => {
                let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
                    values.iter().map(run).collect();
                output_result.map(ScenarioOutput::flatten)
            }
            Some(other) => Err(ScenarioRuntimeError::from(ForEachError::WrongValueType(
                other.clone(),
            ))),
            None => Err(ScenarioRuntimeError::from(ForEachError::NoValueParam)),
        }
    }
}

#[derive(Debug)]
pub enum ForEachError {
    WrongValueType(Value),
    NoValueParam,
}

impl From<ForEachError> for ScenarioRuntimeError {
    fn from(error: ForEachError) -> Self {
        ScenarioRuntimeError::CustomNodeError(Box::new(error))
    }
}

impl Display for ForEachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoValueParam => write!(f, "Parameter 'value' not found"),
            Self::WrongValueType(other) => {
                write!(f, "Parameter 'value' is of wrong type {}", other)
            }
        }
    }
}
impl Error for ForEachError {}
