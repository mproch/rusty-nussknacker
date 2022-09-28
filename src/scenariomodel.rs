use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;

pub fn parse_file(scenario_filename: &Path) -> Result<Scenario, io::Error> {
    let scenario_json = fs::read_to_string(scenario_filename)?;
    parse(&scenario_json)
}

pub fn parse(scenario: &str) -> Result<Scenario, io::Error> {
    let scenario = serde_json::from_str::<Scenario>(scenario)?;
    Ok(scenario)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
///The structure is the same as in Nussknacker project, as the idea is to run (simple) scenarios in Rust without changes.
///The model is a bit simpler, as this is not full-fledged project...
///In particular, joins/unions are not possible
pub enum Node {
    Filter {
        id: NodeId,
        expression: Expression,
    },
    Source {
        id: NodeId,
    },
    Switch {
        id: NodeId,
        nexts: Vec<Case>,
    },
    Split {
        id: NodeId,
        nexts: Vec<Vec<Node>>,
    },
    Sink {
        id: NodeId,
    },
    Variable {
        id: NodeId,
        #[serde(rename = "varName")]
        var_name: String,
        expression: Expression,
    },
    CustomNode {
        id: NodeId,
        #[serde(rename = "outputVar")]
        output_var: String,
        #[serde(rename = "nodeType")]
        node_type: String,
        parameters: Vec<Parameter>,
    },
    //Not implemented at the moment, can be expressed with CustomNode anyway...
    // Enricher {
    //     id: NodeId,
    //     output: String,
    //     service_ref: ServiceRef,
    // },
}

//
impl Node {
    pub fn id(&self) -> &NodeId {
        match self {
            Node::Filter { id, expression: _ } => id,
            Node::Switch { id, nexts: _ } => id,
            Node::Split { id, nexts: _ } => id,
            Node::Variable {
                id,
                var_name: _,
                expression: _,
            } => id,
            Node::CustomNode {
                id,
                output_var: _,
                node_type: _,
                parameters: _,
            } => id,
            Node::Source { id } => id,
            Node::Sink { id } => id,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServiceRef {
    pub id: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub expression: Expression,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Case {
    pub expression: Expression,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Expression {
    pub language: String,
    pub expression: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: &str) -> NodeId {
        NodeId(String::from(id))
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub meta_data: MetaData,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetaData {
    pub id: String,
}
