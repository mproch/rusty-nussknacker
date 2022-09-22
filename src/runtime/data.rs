use serde_json::Value;
use std::collections::HashMap;

pub struct InputData(pub HashMap<String, VarValue>);

pub struct OutputData(pub Vec<VarValue>);

/* 
At the moment we assume JSON model. It's certainly simplification, but for the purpose of this excerise it should be enough;
*/
pub type VarValue = Value;

pub type ScenarioInterpeter = fn(&InputData) -> Result<OutputData, ScenarioError>;

/* 
We leave possiblity of typing variables, but for now we'll be only interested in variable presence, as it makes
JS evaluation simpler.
 */
pub type Type = ();

#[derive(Clone)]
pub struct VarContext(pub HashMap<String, Type>);


#[derive(Debug)]
pub enum ScenarioError {
    ScenarioCompilationError( String),
    ScenarioRuntimeError(String)
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::error::Error for ScenarioError {}