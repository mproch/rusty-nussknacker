use crate::{data::jsonmodel::{Scenario, Node, Node::*, Expression, Case, Parameter}, runtime::data::{OutputData, InputData}, expression::{CompiledExpression, LanguageParser}, customnodes::ForEach};
use serde_json::Value::Bool;
use super::{data::{ScenarioError::{*, self}, VarContext, VarValue}, CustomNodeImpl};
use std::{collections::HashMap, rc::Rc};

pub struct Compiler {
    custom_nodes: HashMap<String, Rc<dyn CustomNodeImpl>>,
    parser: LanguageParser
}

impl Default for Compiler {
    fn default() -> Compiler {
        let for_each: Rc<dyn CustomNodeImpl> = Rc::new(ForEach);
        return Compiler { custom_nodes: HashMap::from([(String::from("forEach"), for_each)]), parser: Default::default() }
    }
}

impl Compiler {


    pub fn compile(&self, scenario: &Scenario) -> Result<Box<dyn Interpreter>, ScenarioError> {
        let iter = &scenario.nodes;
        let initial_input = VarContext(HashMap::from([(String::from("input"), ())]));
        return match iter.first() {
            Some(Source { id: _ }) => self.compile_next(&iter[1..], &initial_input),
            _ => Err(ScenarioCompilationError(String::from("The first node has to be source")))
        };
    }
    
    fn compile_next(&self, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        //TODO: handle empty...
        let rest = &iter[1..];
        match iter.first() {
            Some(Filter { id: _, expression }) => self.compile_filter(expression, rest, var_names),
            Some(Variable { id: _, var_name, expression }) => self.compile_variable(var_name, expression, rest, var_names),
            Some(Switch { id: _, nexts }) => self.compile_switch(nexts, var_names),
            Some(Split { id: _, nexts}) => self.compile_split(nexts, var_names),
            Some(Sink { id: _ }) => Ok(Box::new(CompiledSink {})),
            Some(CustomNode { id: _, output_var, node_type, parameters }) => self.compile_custom_node(output_var, node_type, parameters, rest, var_names),
            Some(_) => Err(ScenarioCompilationError(format!("Unknown node"))),
            None => Err(ScenarioCompilationError(String::from("Invalid end"))),
        }
    }
    
    fn compile_custom_node(&self, output_var: &str, node_type: &str, parameters: &Vec<Parameter>, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        let implementation : &Rc<dyn CustomNodeImpl> = self.custom_nodes.get(node_type).ok_or(ScenarioCompilationError(String::from("Unknown CustomNode")))?;
    
        let next_part = self.compile_next(iter, &var_names.with_var(output_var))?;
        let compiled_parameters: Result<HashMap<String, Box<dyn CompiledExpression>>, ScenarioError> = parameters.iter()
            .map(|p| self.compile_parameter(p, var_names)).collect();
        return Ok(Box::new(CompiledCustomNode { rest: next_part, output_var: String::from(output_var), 
            params: compiled_parameters?, custom_node: implementation.clone()}));
    }

    fn compile_parameter(&self, parameter: &Parameter, var_names: &VarContext) -> Result<(String, Box<dyn CompiledExpression>), ScenarioError> {
        let compiled_expression = self.parser.parse(&parameter.expression, &var_names)?;
        return Ok((parameter.name.clone(), compiled_expression));
    }
    
    fn compile_variable(&self, var_name: &str, raw_expression: &Expression, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        let expression = self.parser.parse(raw_expression, var_names)?;
        let rest = self.compile_next(iter, &var_names.with_var(var_name))?;
        let res = CompiledVariable { rest, expression, var_name: String::from(var_name) };
        return Ok(Box::new(res));
    }
    
    fn compile_filter(&self, raw_expression: &Expression, iter: &[Node], var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        let rest = self.compile_next(iter, var_names)?;
        let expression = self.parser.parse(raw_expression, &var_names)?;
        let res = CompiledFilter { rest, expression };
        return Ok(Box::new(res));
    }
    
    fn compile_switch(&self, nexts: &Vec<Case>, var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        fn parse_case(sself: &Compiler, case: &Case, internal_names: &VarContext) -> Result<CompiledCase, ScenarioError> {
            let rest = sself.compile_next(&case.nodes[..], internal_names)?;
            let expression = sself.parser.parse(&case.expression, &internal_names)?;
            return Ok(CompiledCase { rest, expression })
        
        }
        let compiled: Vec<CompiledCase> = nexts.iter().map(|n| parse_case(self, &n, var_names).unwrap()).collect();
        return Ok(Box::new(CompiledSwitch { nexts: compiled }));
    }
    
    fn compile_split(&self, nexts: &Vec<Vec<Node>>, var_names: &VarContext) -> Result<Box<dyn Interpreter>, ScenarioError> {
        let compiled: Vec<Box<dyn Interpreter>> = nexts.iter().map(|n| self.compile_next(&n[..], var_names).unwrap()).collect();
        return Ok(Box::new(CompiledSplit { nexts: compiled }));
    }
    
}

pub trait Interpreter {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError>;
}

struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String
}

impl Interpreter for CompiledVariable {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        let result = self.expression.execute(&data)?;
        let with_var = data.insert(&self.var_name, result);
        return self.rest.run(&with_var);
    }
}

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>
}


impl Interpreter for CompiledSplit {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        let output_result: Result<Vec<OutputData>, ScenarioError> = self.nexts.iter().map(|one| one.run(data)).collect();
        return output_result.map(OutputData::flatten);
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
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        let mut result: Result<OutputData, ScenarioError> = Ok(OutputData(vec![]));
        for case in &self.nexts  {
            let next_expression = case.expression.execute(data)?;
            let matches = (match next_expression {
               Bool(value) => Ok(value),
               other => Err(ScenarioRuntimeError(format!("Bad switch type: {}", other)))  
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
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        let result = self.expression.execute(data)?;
        return match result {
            Bool(true) => self.rest.run(data),
            Bool(false) => Ok(OutputData(vec![])),
            other => Err(ScenarioRuntimeError(format!("Bad error type: {}", other)))  
        }
    }
}

struct CompiledCustomNode {
    rest: Box<dyn Interpreter>,
    output_var: String, 
    params: HashMap<String, Box<dyn CompiledExpression>>,
    custom_node: Rc<dyn CustomNodeImpl>
}

impl Interpreter for CompiledCustomNode {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        let parameters: Result<HashMap<String, VarValue>, ScenarioError> = 
            self.params.iter().map(|e| e.1.execute(data).map(|r| (String::from(e.0), r))).collect();
        return self.custom_node.run(&self.output_var, parameters?, data, &self.rest);    
    }
}

struct CompiledSink {}

impl Interpreter for CompiledSink {
    fn run(&self, data: &InputData) -> Result<OutputData, ScenarioError> {
        return Ok(OutputData(vec![serde_json::to_value(data.to_serialize()).unwrap()]));
    }
}