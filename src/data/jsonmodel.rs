use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Filter { id: String, expression: Expression },
    Source { id: String },
    Switch { id: String, nexts: Vec<Case> },
    Split { id: String, nexts: Vec<Vec<Node>> },
    Sink { id: String },
    Variable { id: String, varName: String, expression: Expression },

}

#[derive(Serialize, Deserialize)]
pub struct ServiceRef {

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
#[allow(non_snake_case)]
pub struct Scenario {
    pub metaData: MetaData,
    pub nodes: Vec<Node>,
    pub additionalBranches: Vec<Vec<Node>>
}

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    pub id: String
}

#[allow(dead_code)]
enum NodeData {
    Source,
    Sink
}

/*
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
*/