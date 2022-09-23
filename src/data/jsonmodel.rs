use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
//#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Node {
    Filter { id: String, expression: Expression },
    Source { id: String },
    Switch { id: String, nexts: Vec<Case> },
    Split { id: String, nexts: Vec<Vec<Node>> },
    Sink { id: String },
    Variable { id: String, 
        #[serde(rename = "varName")]
        var_name: String, expression: Expression },
    Enricher { id: String, output: String, service_ref: ServiceRef },
    CustomNode { id: String, 
        #[serde(rename = "outputVar")]
        output_var: String, 
        #[serde(rename = "nodeType")]
        node_type: String, parameters: Vec<Parameter> }
}

#[derive(Serialize, Deserialize)]
pub struct ServiceRef {
    pub id: String,
    pub parameters: Vec<Parameter>
}

#[derive(Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    expression: Expression
}

#[derive(Serialize, Deserialize)]
pub struct Case {
    pub expression: Expression,
    pub nodes: Vec<Node>
}

#[derive(Serialize, Deserialize)]
pub struct Expression {
    pub language: String,
    pub expression: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub meta_data: MetaData,
    pub nodes: Vec<Node>,
    pub additional_branches: Vec<Vec<Node>>
}

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    pub id: String
}
