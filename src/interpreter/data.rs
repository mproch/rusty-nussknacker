use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, rc::Rc};

/// Data passed through scenario
/// We keep Rc<VarValue> as value in map to avoid excessive cloning. 
#[derive(Clone)]
pub struct VarContext(pub HashMap<String, Rc<VarValue>>);

impl VarContext {
    pub fn default_input(value: Value) -> VarContext {
        VarContext(HashMap::from([(String::from("input"), Rc::new(value))]))
    }
    pub fn to_external_form(&self) -> HashMap<String, VarValue> {
        return self.0.iter().map(|f| (f.0.clone(), f.1.as_ref().to_owned())).collect();
    }
    pub fn insert(&self, name: &str, value: Value) -> VarContext {
        let mut result = self.clone();
        result.0.insert(String::from(name), Rc::new(value));
        result
    }
}

///Output data of the scenario
///The data may reach many sinks (e.g. after split) or none (e.g. after filter)
#[derive(Serialize, PartialEq, Debug)]
pub struct ScenarioOutput(pub Vec<SingleScenarioOutput>);

impl ScenarioOutput {
    pub fn flatten(vec: Vec<ScenarioOutput>) -> ScenarioOutput {
        ScenarioOutput(vec.into_iter().flat_map(|o| o.0).collect())   
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct SingleScenarioOutput {
    pub node_id: String,
    pub variables: HashMap<String, VarValue>
}

/// 
/// At the moment we assume JSON model. It's certainly simplification, but for the purpose of this excerise it should be enough;
pub type VarValue = Value;

/* 
We leave possiblity of typing variables, but for now we'll be only interested in variable presence, as it makes
JS evaluation simpler.
 */
pub type VarType = ();

#[derive(Clone)]
pub struct CompilationVarContext(pub HashMap<String, VarType>);

impl CompilationVarContext {
    pub fn with_var(&self, name: &str) -> CompilationVarContext {
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