use crate::{scenariomodel::{Scenario, Node, Node::*, Expression, Case, Parameter}, runtime::data::{ScenarioOutput, VarContext}, expression::{CompiledExpression, LanguageParser}, customnodes::ForEach};
use serde_json::Value::Bool;
use super::{data::{ScenarioRuntimeError, ScenarioCompilationError, CompilationVarContext, VarValue}, CustomNodeImpl, Interpreter};
use std::{collections::HashMap, rc::Rc};

pub struct Compiler {
    custom_nodes: HashMap<String, Rc<dyn CustomNodeImpl>>,
    parser: LanguageParser
}

impl Default for Compiler {
    fn default() -> Compiler {
        let for_each: Rc<dyn CustomNodeImpl> = Rc::new(ForEach);
        Compiler { custom_nodes: HashMap::from([(String::from("forEach"), for_each)]), parser: Default::default() }
    }
}

impl Compiler {

    pub fn compile(&self, scenario: &Scenario) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let iter = &scenario.nodes;
        let initial_input = CompilationVarContext(HashMap::from([(String::from("input"), ())]));
        return match iter.first() {
            Some(Source { id: _ }) => self.compile_next(&iter[1..], &initial_input),
            _ => Err(ScenarioCompilationError(String::from("The first node has to be source")))
        };
    }
    
    fn compile_next(&self, iter: &[Node], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        match iter.first() {
            Some(first) => self.compile_next_node(first, &iter[1..], var_names),
            None => Err(ScenarioCompilationError(String::from("Invalid end"))),
        }
    }

    fn compile_next_node(&self, head: &Node, rest: &[Node], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        match head {
            Filter { id: _, expression } => self.compile_filter(expression, rest, var_names),
            Variable { id: _, var_name, expression } => self.compile_variable(var_name, expression, rest, var_names),
            Switch { id: _, nexts } => self.compile_switch(nexts, var_names),
            Split { id: _, nexts} => self.compile_split(nexts, var_names),
            Sink { id: _ } => Ok(Box::new(CompiledSink {})),
            CustomNode { id: _, output_var, node_type, parameters } => self.compile_custom_node(output_var, node_type, parameters, rest, var_names),
            _ => Err(ScenarioCompilationError("Unknown node".to_string())),
        }
    }
    
    fn compile_custom_node(&self, output_var: &str, node_type: &str, parameters: &[Parameter], iter: &[Node], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let implementation : &Rc<dyn CustomNodeImpl> = self.custom_nodes.get(node_type).ok_or_else(|| ScenarioCompilationError(String::from("Unknown CustomNode")))?;
    
        let next_part = self.compile_next(iter, &var_names.with_var(output_var))?;
        let compiled_parameters: Result<HashMap<String, Box<dyn CompiledExpression>>, ScenarioCompilationError> = parameters.iter()
            .map(|p| self.compile_parameter(p, var_names)).collect();
        Ok(Box::new(CompiledCustomNode { rest: next_part, output_var: String::from(output_var), 
            params: compiled_parameters?, custom_node: implementation.clone()}))
    }

    fn compile_parameter(&self, parameter: &Parameter, var_names: &CompilationVarContext) -> Result<(String, Box<dyn CompiledExpression>), ScenarioCompilationError> {
        let compiled_expression = self.parser.parse(&parameter.expression, var_names)?;
        Ok((parameter.name.clone(), compiled_expression))
    }
    
    fn compile_variable(&self, var_name: &str, raw_expression: &Expression, iter: &[Node], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let expression = self.parser.parse(raw_expression, var_names)?;
        let rest = self.compile_next(iter, &var_names.with_var(var_name))?;
        let res = CompiledVariable { rest, expression, var_name: String::from(var_name) };
        Ok(Box::new(res))
    }
    
    fn compile_filter(&self, raw_expression: &Expression, iter: &[Node], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let rest = self.compile_next(iter, var_names)?;
        let expression = self.parser.parse(raw_expression, var_names)?;
        let res = CompiledFilter { rest, expression };
        Ok(Box::new(res))
    }
    
    fn compile_switch(&self, nexts: &[Case], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        fn parse_case(sself: &Compiler, case: &Case, internal_names: &CompilationVarContext) -> Result<CompiledCase, ScenarioCompilationError> {
            let rest = sself.compile_next(&case.nodes[..], internal_names)?;
            let expression = sself.parser.parse(&case.expression, internal_names)?;
            Ok(CompiledCase { rest, expression })
        
        }
        let compiled: Vec<CompiledCase> = nexts.iter().map(|n| parse_case(self, n, var_names).unwrap()).collect();
        Ok(Box::new(CompiledSwitch { nexts: compiled }))
    }
    
    fn compile_split(&self, nexts: &[Vec<Node>], var_names: &CompilationVarContext) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let compiled: Vec<Box<dyn Interpreter>> = nexts.iter().map(|n| self.compile_next(&n[..], var_names).unwrap()).collect();
        Ok(Box::new(CompiledSplit { nexts: compiled }))
    }
    
}

struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String
}

impl Interpreter for CompiledVariable {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        let with_var = data.insert(&self.var_name, result);
        self.rest.run(&with_var)
    }
}

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>
}


impl Interpreter for CompiledSplit {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> = self.nexts.iter().map(|one| one.run(data)).collect();
        output_result.map(ScenarioOutput::flatten)
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
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let mut result: Result<ScenarioOutput, ScenarioRuntimeError> = Ok(ScenarioOutput(vec![]));
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
        result
    }
}

struct CompiledFilter {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>
}


impl Interpreter for CompiledFilter {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        match result {
            Bool(true) => self.rest.run(data),
            Bool(false) => Ok(ScenarioOutput(vec![])),
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
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let parameters: Result<HashMap<String, VarValue>, ScenarioRuntimeError> = 
            self.params.iter().map(|e| e.1.execute(data).map(|r| (String::from(e.0), r))).collect();
        self.custom_node.run(&self.output_var, parameters?, data, self.rest.as_ref())   
    }
}

struct CompiledSink {}

impl Interpreter for CompiledSink {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        Ok(ScenarioOutput(vec![serde_json::to_value(data.to_serialize()).unwrap()]))
    }
}