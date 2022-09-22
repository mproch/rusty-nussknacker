use crate::{data::jsonmodel::{Scenario, Node, Node::*, Expression, Case}, runtime::data::{OutputData, InputData}, expression::CompiledExpression};
use serde_json::Value::Bool;
use super::data::{ScenarioError, VarContext};
use core::slice::Iter;
use std::collections::HashMap;

pub fn compile(scenario: &Scenario) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let iter = scenario.nodes.iter();
    
    return match iter.next() {
        Some(Source { id }) => compile_next(iter),
        _ => todo!("")
    };
}

fn compile_next(iter: Iter<Node>) -> Result<Box<dyn Interpreter>, ScenarioError> {
    match iter.next() {
        Some(Filter { id, expression }) => compile_filter(expression, iter),
        Some(Variable { id, varName, expression }) => compile_variable(varName, expression, iter),
        Some(Switch { id, nexts }) => compile_switch(nexts),
        Some(Split { id, nexts}) => compile_split(nexts),
        Some(Sink { id }) => Ok(Box::new(CompiledSink {})),
        _ => todo!("")
    }
}

fn compile_variable(var_name: &String, raw_expression: &Expression, iter: Iter<Node>) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let rest = compile_next(iter)?;
    let expression = crate::expression::parse::parse(raw_expression, VarContext(HashMap::from([])))?;
    //??clone
    let res = CompiledVariable { rest, expression, var_name: var_name.clone() };
    return Ok(Box::new(res));
}

fn compile_filter(raw_expression: &Expression, iter: Iter<Node>) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let rest = compile_next(iter)?;
    let expression = crate::expression::parse::parse(raw_expression, VarContext(HashMap::from([])))?;
    let res = CompiledFilter { rest, expression };
    return Ok(Box::new(res));
}

fn compile_switch(nexts: &Vec<Case>) -> Result<Box<dyn Interpreter>, ScenarioError> {
    fn parse_case(case: &Case) -> Result<CompiledCase, ScenarioError> {
        let rest = compile_next(case.nodes.iter())?;
        let expression = crate::expression::parse::parse(&case.expression, VarContext(HashMap::from([])))?;
        return Ok(CompiledCase { rest, expression })
    
    }
    let compiled: Vec<CompiledCase> = nexts.iter().map(|n| parse_case(&n).unwrap()).collect();
    return Ok(Box::new(CompiledSwitch { nexts: compiled }));
}

fn compile_split(nexts: &Vec<Vec<Node>>) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let compiled: Vec<Box<dyn Interpreter>> = nexts.iter().map(|n| compile_next(n.iter()).unwrap()).collect();
    return Ok(Box::new(CompiledSplit { nexts: compiled }));
}


pub trait Interpreter {
    fn run(&self, data: &mut InputData) -> Result<OutputData, ScenarioError>;
}


struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String
}

impl Interpreter for CompiledVariable {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        let result = self.expression.execute(&data)?;
        data.0.insert(self.var_name, result);
        return self.rest.run(data);
    }
}

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>
}


impl Interpreter for CompiledSplit {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        let res: Vec<Result<OutputData, ScenarioError>> = self.nexts.iter().map(|one| one.run(data)).collect();
        todo!("")
    }
}

struct CompiledSwitch {
    nexts: Vec<CompiledCase>
}

struct CompiledCase {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>
}

impl Interpreter for CompiledSwitch {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        let mut result: Result<OutputData, ScenarioError> = Ok(OutputData(vec![]));
        for case in self.nexts  {
            let nextExpression = case.expression.execute(data)?;
            let matches = (match nextExpression {
               Bool(value) => Ok(value),
                _ => Err(ScenarioError(String::from("Bad error type")))  
            })?;
            if matches {
                result = case.rest.run(data);
                break;
            }          
        }
        return result;
    }
}

struct CompiledFilter {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>
}


impl Interpreter for CompiledFilter {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        let result = self.expression.execute(data)?;
        return match result {
            Bool(value) if value => self.rest.run(data),
            Bool(value) if !value => Ok(OutputData(vec![])),
            _ => Err(ScenarioError(String::from("Bad error type")))  
        }
    }
}

struct CompiledSink {}

impl Interpreter for CompiledSink {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        return Ok(OutputData(vec![serde_json::to_value(&data.0).unwrap()]));
    }
}