use crate::{data::jsonmodel::{Scenario, Node, Node::*, Expression, Case}, runtime::data::{OutputData, InputData}, expression::CompiledExpression};
use serde_json::Value::Bool;
use super::data::{ScenarioError::{*, self}, VarContext, VarValue};
use core::slice::Iter;
use std::collections::HashMap;

pub fn compile(scenario: &Scenario) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let iter = &scenario.nodes;
    let initial_input = VarContext(HashMap::from([(String::from("input"), ())]));
    return match iter.first() {
        Some(Source { id }) => compile_next(&iter[1..], &initial_input),
        _ => todo!("")
    };
}

fn compile_next(iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let rest = &iter[1..];
    match iter.first() {
        Some(Filter { id, expression }) => compile_filter(expression, rest, var_names),
        Some(Variable { id, varName, expression }) => compile_variable(varName.to_string(), expression, rest, var_names),
        Some(Switch { id, nexts }) => compile_switch(nexts, var_names),
        Some(Split { id, nexts}) => compile_split(nexts, var_names),
        Some(Sink { id }) => Ok(Box::new(CompiledSink {})),
        _ => todo!("")
    }
}

fn compile_variable(var_name: String, raw_expression: &Expression, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let expression = crate::expression::parse::parse(raw_expression, &var_names)?;
    let mut new_names = var_names.clone();
    //??clone
    new_names.0.insert(var_name.clone(), ());
    let rest = compile_next(iter, &new_names)?;

    let res = CompiledVariable { rest, expression, var_name: var_name };
    return Ok(Box::new(res));
}

fn compile_filter(raw_expression: &Expression, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let rest = compile_next(iter, var_names)?;
    let expression = crate::expression::parse::parse(raw_expression, &var_names)?;
    let res = CompiledFilter { rest, expression };
    return Ok(Box::new(res));
}

fn compile_switch(nexts: &Vec<Case>, var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
    fn parse_case(case: &Case, internal_names: &VarContext) -> Result<CompiledCase, ScenarioError> {
        let rest = compile_next(&case.nodes[..], internal_names)?;
        let expression = crate::expression::parse::parse(&case.expression, &internal_names)?;
        return Ok(CompiledCase { rest, expression })
    
    }
    let compiled: Vec<CompiledCase> = nexts.iter().map(|n| parse_case(&n, var_names).unwrap()).collect();
    return Ok(Box::new(CompiledSwitch { nexts: compiled }));
}

fn compile_split(nexts: &Vec<Vec<Node>>, var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
    let compiled: Vec<Box<dyn Interpreter>> = nexts.iter().map(|n| compile_next(&n[..], var_names).unwrap()).collect();
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
        data.0.insert(self.var_name.clone(), result);
        return self.rest.run(data);
    }
}

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>
}


impl Interpreter for CompiledSplit {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        let res: Result<Vec<OutputData>, ScenarioError> = self.nexts.iter().map(|one| one.run(data)).collect();
        //TODO: remove clone??
        let flattened: Vec<VarValue> = (res?).iter().map(|o| o.0.clone()).flatten().collect();
        return Ok(OutputData(flattened));
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
        for case in &self.nexts  {
            let next_expression = case.expression.execute(data)?;
            let matches = (match next_expression {
               Bool(value) => Ok(value),
                _ => Err(ScenarioRuntimeError(String::from("Bad error type")))  
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
            _ => Err(ScenarioRuntimeError(String::from("Bad error type")))  
        }
    }
}

struct CompiledSink {}

impl Interpreter for CompiledSink {
    fn run(&self, data: & mut InputData) -> Result<OutputData, ScenarioError> {
        return Ok(OutputData(vec![serde_json::to_value(&data.0).unwrap()]));
    }
}