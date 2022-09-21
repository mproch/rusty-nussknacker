use crate::{data::jsonmodel::Scenario, runtime::data::{OutputData, InputData}};

use super::data::{ScenarioInterpeter, ScenarioError};

pub fn compile(_scenario: Scenario) -> Result<ScenarioInterpeter, ScenarioError> {
    
    let res = |_input: InputData| { panic!("bla") };
    Ok(res)
}


trait NodeExecution {
    fn run(&self, data: InputData, rest: dyn NodeExecution) -> OutputData;
}


