use serde_json::Value;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone)]
pub struct InputData(pub HashMap<String, Rc<VarValue>>);

impl InputData {
    pub fn default_input(value: Value) -> InputData {
        InputData(HashMap::from([(String::from("input"), Rc::new(value))]))
    }
    pub fn to_serialize(&self) -> HashMap<String, &VarValue> {
        return self.0.iter().map(|f| (f.0.clone(), f.1.as_ref())).collect();
    }
    pub fn insert(&self, name: &str, value: Value) -> InputData {
        let mut with_new = self.clone();
        with_new.0.insert(String::from(name), Rc::new(value));
        with_new
    }
}

pub struct OutputData(pub Vec<VarValue>);

impl OutputData {
    pub fn flatten(vec: Vec<OutputData>) -> OutputData {
        OutputData(vec.into_iter().flat_map(|o| o.0).collect())   
    }
}

/* 
At the moment we assume JSON model. It's certainly simplification, but for the purpose of this excerise it should be enough;
*/
pub type VarValue = Value;

pub type ScenarioInterpeter = fn(&InputData) -> Result<OutputData, ScenarioRuntimeError>;

/* 
We leave possiblity of typing variables, but for now we'll be only interested in variable presence, as it makes
JS evaluation simpler.
 */
pub type Type = ();

#[derive(Clone)]
pub struct VarContext(pub HashMap<String, Type>);

impl VarContext {
    pub fn with_var(&self, name: &str) -> VarContext {
        let mut new_ctx = self.clone();
        new_ctx.0.insert(String::from(name), ());
        new_ctx
    }
}

#[derive(Debug)]
pub struct ScenarioCompilationError(pub String);

#[derive(Debug)]
pub struct ScenarioRuntimeError(pub String);

impl std::fmt::Display for ScenarioCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ScenarioCompilationError {}

impl std::fmt::Display for ScenarioRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ScenarioRuntimeError {}