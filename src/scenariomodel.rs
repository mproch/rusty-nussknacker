use serde::Deserialize;
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

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Filter {
        id: String,
        expression: Expression,
    },
    Source {
        id: String,
    },
    Switch {
        id: String,
        nexts: Vec<Case>,
    },
    Split {
        id: String,
        nexts: Vec<Vec<Node>>,
    },
    Sink {
        id: String,
    },
    Variable {
        id: String,
        #[serde(rename = "varName")]
        var_name: String,
        expression: Expression,
    },
    CustomNode {
        id: String,
        #[serde(rename = "outputVar")]
        output_var: String,
        #[serde(rename = "nodeType")]
        node_type: String,
        parameters: Vec<Parameter>,
    },
    //Not implemented at the moment ;)
    Enricher {
        id: String,
        output: String,
        service_ref: ServiceRef,
    },
}

#[derive(Deserialize)]
pub struct ServiceRef {
    pub id: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Deserialize)]
pub struct Parameter {
    pub name: String,
    pub expression: Expression,
}

#[derive(Deserialize)]
pub struct Case {
    pub expression: Expression,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize)]
pub struct Expression {
    pub language: String,
    pub expression: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub meta_data: MetaData,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize)]
pub struct MetaData {
    pub id: String,
}
