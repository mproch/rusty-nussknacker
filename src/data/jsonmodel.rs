use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
    CustomNode { id: String, 
        #[serde(rename = "outputVar")]
        output_var: String, 
        #[serde(rename = "nodeType")]
        node_type: String, parameters: Vec<Parameter> },
    //Not implemented at the moment ;) 
    Enricher { id: String, output: String, service_ref: ServiceRef }
}

#[derive(Serialize, Deserialize)]
pub struct ServiceRef {
    pub id: String,
    pub parameters: Vec<Parameter>
}

#[derive(Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub expression: Expression
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
