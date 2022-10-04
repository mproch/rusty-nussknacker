use crate::interpreter::{
    data::{ScenarioOutput, ScenarioRuntimeError, VarContext, VarValue},
    CustomNodeImpl, Interpreter,
};
use serde_json::Value::{self, Array};
use std::{collections::HashMap, error::Error, fmt::Display};
pub struct ForEach;

const VALUE_PARAM: &str = "value";

///The components requires "value" parameter of array type.
///For each element of array, the subsequent part of the scenario is invoked, with the element passed as an output variable.
///This is the implementation of: https://nussknacker.io/documentation/docs/scenarios_authoring/BasicNodes#foreach
impl CustomNodeImpl for ForEach {
    fn run(
        &self,
        output_var: &str,
        parameters: &HashMap<String, VarValue>,
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

#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use crate::{
        customnodes::ForEachError,
        interpreter::{
            data::{
                ScenarioOutput, ScenarioRuntimeError, SingleScenarioOutput, VarContext, VarValue,
            },
            CustomNodeImpl, Interpreter,
        },
        scenariomodel::NodeId,
    };
    use serde_json::{json, Value};
    use std::collections::HashMap;

    use super::{ForEach, VALUE_PARAM};

    const TEST_OUTPUT: &str = "test_output";

    struct MockInterpreter {}

    impl Interpreter for MockInterpreter {
        fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
            Ok(ScenarioOutput(vec![SingleScenarioOutput {
                node_id: NodeId::new(TEST_OUTPUT),
                variables: data.to_external_form(),
            }]))
        }
    }

    #[test]
    fn test_arrays() -> Result<(), ScenarioRuntimeError> {
        let foreach = ForEach {};
        let output_var = "output";
        let next_part = MockInterpreter {};

        let check_for_value = |v: &[&VarValue]| -> Result<(), ScenarioRuntimeError> {
            let parameters = HashMap::from([(VALUE_PARAM.to_owned(), json!(v))]);
            let result = foreach.run(output_var, &parameters, &VarContext::empty(), &next_part)?;
            let values: Vec<&VarValue> = result
                .var_in_sink(&NodeId::new(TEST_OUTPUT), output_var)
                .iter()
                .map(|o| o.unwrap())
                .collect();
            assert_eq!(values, v);
            Ok(())
        };
        check_for_value(&[&json!("a"), &json!(12)])?;
        check_for_value(&[])?;
        check_for_value(&[&json!(""), &Value::Null])?;

        Ok(())
    }

    #[test]
    fn test_wrong_parameters() {
        let foreach = ForEach {};
        let next_part = MockInterpreter {};
        let output_var = "output";

        let test_parameter = |params: &HashMap<String, VarValue>, expected_error: ForEachError| {
            let result = foreach
                .run(output_var, params, &VarContext::empty(), &next_part)
                .unwrap_err();
            let error = match result {
                ScenarioRuntimeError::CustomNodeError(error) => {
                    error.downcast::<ForEachError>().unwrap()
                }
                other => panic!("Unexpected error {:?}", other),
            };
            assert_eq!(*error, expected_error);
        };
        test_parameter(&HashMap::from([]), ForEachError::NoValueParam);
        test_parameter(
            &HashMap::from([(String::from("dummy_name"), json!([]))]),
            ForEachError::NoValueParam,
        );

        test_parameter(
            &HashMap::from([(String::from(VALUE_PARAM), json!(""))]),
            ForEachError::WrongValueType(json!("")),
        );
    }
}
