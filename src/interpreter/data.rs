use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

use crate::scenariomodel::{Node, NodeId};
use once_cell::sync::Lazy;

/// Data passed through scenario
/// We keep Arc<VarValue> as value in map to avoid excessive cloning.
#[derive(Clone)]
pub struct VarContext(HashMap<String, Arc<VarValue>>);

pub const DEFAULT_INPUT_NAME: &str = "input";

impl VarContext {
    pub fn empty() -> VarContext {
        VarContext(HashMap::from([]))
    }

    pub fn default_context_for_value(value: Value) -> VarContext {
        VarContext(HashMap::from([(
            DEFAULT_INPUT_NAME.to_string(),
            Arc::new(value),
        )]))
    }
    //this is mainly for computing ScenarioOutput and for passing to expressions
    pub fn to_external_form(&self) -> HashMap<String, VarValue> {
        return self
            .0
            .iter()
            //not quite sure if all this is needed
            .map(|f| (f.0.clone(), f.1.as_ref().to_owned()))
            .collect();
    }
    pub fn with_new_var(&self, name: &str, value: Value) -> VarContext {
        let mut result = self.clone();
        result.0.insert(String::from(name), Arc::new(value));
        result
    }
}

///Output data of the scenario
///The data may reach many sinks (e.g. after split) or none (e.g. after filter)
#[derive(Serialize, PartialEq, Eq, Debug)]
pub struct ScenarioOutput(pub Vec<SingleScenarioOutput>);

impl ScenarioOutput {
    pub fn flatten(vec: Vec<ScenarioOutput>) -> ScenarioOutput {
        ScenarioOutput(vec.into_iter().flat_map(|o| o.0).collect())
    }

    pub fn vars_in_sink(&self, sink_id: &NodeId) -> Vec<&HashMap<String, Value>> {
        self.0
            .iter()
            .filter(|out| out.node_id == *sink_id)
            .map(|out| &out.variables)
            .collect()
    }

    pub fn var_in_sink(&self, sink_id: &NodeId, var_name: &str) -> Vec<Option<&Value>> {
        self.vars_in_sink(sink_id)
            .iter()
            .map(|out| out.get(var_name))
            .collect()
    }
}

#[derive(Serialize, PartialEq, Eq, Debug)]
pub struct SingleScenarioOutput {
    pub node_id: NodeId,
    pub variables: HashMap<String, VarValue>,
}

/// At the moment we assume JSON model. It's certainly a simplification, but for the purpose of this excerise it should be enough;
pub type VarValue = Value;

/*
We leave possiblity of typing variables, but for now we'll be only interested in variable presence, as it makes
JS evaluation simpler.
 */
pub type VarType = ();

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompilationVarContext(pub HashMap<String, VarType>);

static VAR_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new("^[a-z][a-z0-9_]*$").unwrap());

impl CompilationVarContext {
    pub fn default() -> CompilationVarContext {
        CompilationVarContext(HashMap::from([(DEFAULT_INPUT_NAME.to_string(), ())]))
    }

    pub fn with_var(
        &self,
        node_id: &NodeId,
        name: &str,
    ) -> Result<CompilationVarContext, ScenarioCompilationError> {
        if !VAR_PATTERN.is_match(name) {
            return Err(ScenarioCompilationError::IncorrectVariableName {
                node_id: node_id.clone(),
                var_name: name.to_string(),
            });
        }
        //we clone to be able to pass it freely to different branches e.g. in split.
        let mut new_ctx = self.clone();
        new_ctx.0.insert(String::from(name), ());
        Ok(new_ctx)
    }
}

#[derive(Debug)]
pub enum ScenarioCompilationError {
    IncorrectVariableName {
        node_id: NodeId,
        var_name: String,
    },
    UnknownLanguage {
        node_id: NodeId,
        language: String,
    },
    ScenarioReadFailure(std::io::Error),
    ParseError {
        node_id: NodeId,
        error: Box<dyn crate::expression::ParseError>,
    },
    InvalidEnd(NodeId),
    FirstNodeNotSource(NodeId),
    UnknownNode(NodeId),
    UnknownCustomNode {
        node_id: NodeId,
        node_type: String,
    },
    NodesAfterEndingNode {
        node_id: NodeId,
        unexpected_nodes: Vec<Node>,
    },
    EmptyScenario(),
}

impl std::fmt::Display for ScenarioCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //this should be nicely handled, just like in ForEachError...
        write!(f, "Error occurred: {:?}", self)
    }
}

impl std::error::Error for ScenarioCompilationError {}

#[derive(Debug)]
//TODO: pass NodeId in runtime errors, like in compilation errors
pub enum ScenarioRuntimeError {
    CannotParseInput(serde_json::Error),
    InvalidSwitchType(Value),
    InvalidFilterType(Value),
    ExpressionError(Box<dyn std::error::Error + Send>),
    CustomNodeError(Box<dyn std::error::Error + Send>),
}

impl std::fmt::Display for ScenarioRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //this should be nicely handled, just like in ForEachError...
        write!(f, "Error occurred: {:?}", self)
    }
}

impl std::error::Error for ScenarioRuntimeError {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{interpreter::data::ScenarioCompilationError, scenariomodel::NodeId};

    use super::CompilationVarContext;

    fn node_id() -> NodeId {
        NodeId::new("testNode")
    }

    #[test]
    fn adds_var_to_context() -> Result<(), ScenarioCompilationError> {
        let context = CompilationVarContext::default();
        let new_ctx = context.with_var(&node_id(), "abc")?;
        assert_eq!(
            new_ctx.0,
            HashMap::from([("input".to_string(), ()), ("abc".to_string(), ())])
        );
        Ok(())
    }

    #[test]
    fn checks_var_name() {
        fn assert_incorrent_name(name: &str) {
            let context = CompilationVarContext::default();
            match context.with_var(&node_id(), name) {
                Err(ScenarioCompilationError::IncorrectVariableName {
                    node_id: id,
                    var_name,
                }) if name == var_name && id == node_id() => (),
                other => panic!("Unexpected: {:?}", other),
            }
        }
        assert_incorrent_name("a b c");
        assert_incorrent_name("1abc");
        assert_incorrent_name("");
    }
}
