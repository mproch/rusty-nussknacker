use super::{
    data::{
        CompilationVarContext, ScenarioCompilationError, ScenarioRuntimeError,
        SingleScenarioOutput, VarValue,
    },
    CompilationResult, CustomNodeImpl, Interpreter,
};
use crate::{
    customnodes::ForEach,
    expression::{CompiledExpression, LanguageParser},
    interpreter::data::{ScenarioOutput, VarContext},
    scenariomodel::{Case, Expression, Node, Node::*, Parameter, Scenario},
};
use serde_json::Value::Bool;
use std::{collections::HashMap, rc::Rc};

///The compiler can be customized with additional language runtimes and additional custom components.
/// By default, simple javascript language parser and for-each components are provided
pub struct Compiler {
    custom_nodes: HashMap<String, Rc<dyn CustomNodeImpl>>,
    parser: LanguageParser,
}

impl Default for Compiler {
    fn default() -> Compiler {
        let for_each: Rc<dyn CustomNodeImpl> = Rc::new(ForEach);
        Compiler {
            custom_nodes: HashMap::from([(String::from("forEach"), for_each)]),
            parser: Default::default(),
        }
    }
}

//I'd like to split this code to smaller parts, however my current Rust knowledge doesn't allow me to
//create sth along the lines of:
//
// struct CompilerContext {
//     node_id: String,
//     variables: CompilationVarContext,
//     //in JVM world, I'd have a pointer to compiler::compile_next method, but here I cannot get it working
//     compile_next: Fn(&[Node], &CompilationVarContext) -> CompilationResult
// }
//and then I'd be able to split implementation and tests for each node type
impl Compiler {
    pub fn compile(&self, scenario: &Scenario) -> CompilationResult {
        let iter = &scenario.nodes;
        let initial_input = CompilationVarContext::default();
        return match iter.first() {
            Some(Source { id: _ }) => self.compile_next(&iter[1..], &initial_input),
            _ => Err(ScenarioCompilationError::FirstNodeNotSource()),
        };
    }

    fn compile_next(&self, iter: &[Node], var_names: &CompilationVarContext) -> CompilationResult {
        match iter.first() {
            Some(first) => self.compile_next_node(first, &iter[1..], var_names),
            None => Err(ScenarioCompilationError::InvalidEnd()),
        }
    }

    fn compile_next_node(
        &self,
        head: &Node,
        rest: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        match head {
            Filter { id: _, expression } => self.compile_filter(expression, rest, var_names),
            Variable {
                id: _,
                var_name,
                expression,
            } => self.compile_variable(var_name, expression, rest, var_names),
            Switch { id: _, nexts } => self.compile_switch(nexts, var_names),
            Split { id: _, nexts } => self.compile_split(nexts, var_names),
            Sink { id } => self.compile_sink(id, rest),
            CustomNode {
                id: _,
                output_var,
                node_type,
                parameters,
            } => self.compile_custom_node(output_var, node_type, parameters, rest, var_names),
            _ => Err(ScenarioCompilationError::UnknownNode()),
        }
    }

    fn compile_custom_node(
        &self,
        output_var: &str,
        node_type: &str,
        parameters: &[Parameter],
        iter: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let implementation: &Rc<dyn CustomNodeImpl> = self
            .custom_nodes
            .get(node_type)
            .ok_or_else(|| ScenarioCompilationError::UnknownCustomNode(node_type.to_string()))?;

        let next_part = self.compile_next(iter, &var_names.with_var(output_var)?)?;
        let compiled_parameters: Result<
            HashMap<String, Box<dyn CompiledExpression>>,
            ScenarioCompilationError,
        > = parameters
            .iter()
            .map(|p| self.compile_parameter(p, var_names))
            .collect();
        Ok(Box::new(CompiledCustomNode {
            rest: next_part,
            output_var: String::from(output_var),
            params: compiled_parameters?,
            custom_node: implementation.clone(),
        }))
    }

    fn compile_parameter(
        &self,
        parameter: &Parameter,
        var_names: &CompilationVarContext,
    ) -> Result<(String, Box<dyn CompiledExpression>), ScenarioCompilationError> {
        let compiled_expression = self.parser.parse(&parameter.expression, var_names)?;
        Ok((parameter.name.clone(), compiled_expression))
    }

    //In fact, variable and filter can be implemented as custom nodes
    fn compile_variable(
        &self,
        var_name: &str,
        raw_expression: &Expression,
        iter: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let expression = self.parser.parse(raw_expression, var_names)?;
        let rest = self.compile_next(iter, &var_names.with_var(var_name)?)?;
        let res = CompiledVariable {
            rest,
            expression,
            var_name: String::from(var_name),
        };
        Ok(Box::new(res))
    }

    fn compile_filter(
        &self,
        raw_expression: &Expression,
        iter: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let rest = self.compile_next(iter, var_names)?;
        let expression = self.parser.parse(raw_expression, var_names)?;
        let res = CompiledFilter { rest, expression };
        Ok(Box::new(res))
    }

    fn compile_switch(
        &self,
        nexts: &[Case],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let parse_case = |case: &Case| {
            let rest = self.compile_next(&case.nodes[..], var_names)?;
            let expression = self.parser.parse(&case.expression, var_names)?;
            Ok(CompiledCase { rest, expression })
        };
        let compiled: Result<Vec<CompiledCase>, ScenarioCompilationError> =
            nexts.iter().map(parse_case).collect();
        Ok(Box::new(CompiledSwitch { nexts: compiled? }))
    }

    fn compile_split(
        &self,
        nexts: &[Vec<Node>],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let compiled: Result<Vec<Box<dyn Interpreter>>, ScenarioCompilationError> = nexts
            .iter()
            .map(|n| self.compile_next(&n[..], var_names))
            .collect();
        Ok(Box::new(CompiledSplit { nexts: compiled? }))
    }

    fn compile_sink(&self, sink_id: &str, rest: &[Node]) -> CompilationResult {
        if rest.is_empty() {
            Ok(Box::new(CompiledSink {
                node_id: String::from(sink_id),
            }))
        } else {
            Err(ScenarioCompilationError::NodesAfterSink(rest.to_vec()))
        }
    }
}

struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String,
}

impl Interpreter for CompiledVariable {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        let with_var = data.insert(&self.var_name, result);
        self.rest.run(&with_var)
    }
}

struct CompiledSplit {
    nexts: Vec<Box<dyn Interpreter>>,
}

impl Interpreter for CompiledSplit {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
        //I'm a bit worried that the compiler does not force this clone...
            self.nexts.iter().map(|one| one.run(&data.clone())).collect();
        output_result.map(ScenarioOutput::flatten)
    }
}

struct CompiledSwitch {
    nexts: Vec<CompiledCase>,
}

struct CompiledCase {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
}

impl Interpreter for CompiledSwitch {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let mut result: Result<ScenarioOutput, ScenarioRuntimeError> = Ok(ScenarioOutput(vec![]));
        for case in &self.nexts {
            let next_expression = case.expression.execute(data)?;
            let matches = (match next_expression {
                Bool(value) => Ok(value),
                other => Err(ScenarioRuntimeError::InvalidSwitchType(other)),
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
    expression: Box<dyn CompiledExpression>,
}

impl Interpreter for CompiledFilter {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let result = self.expression.execute(data)?;
        match result {
            Bool(true) => self.rest.run(data),
            Bool(false) => Ok(ScenarioOutput(vec![])),
            other => Err(ScenarioRuntimeError::InvalidSwitchType(other)),
        }
    }
}

struct CompiledCustomNode {
    rest: Box<dyn Interpreter>,
    output_var: String,
    params: HashMap<String, Box<dyn CompiledExpression>>,
    custom_node: Rc<dyn CustomNodeImpl>,
}

impl Interpreter for CompiledCustomNode {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let parameters: Result<HashMap<String, VarValue>, ScenarioRuntimeError> = self
            .params
            .iter()
            //I was hoping for some nice variant of mapValues...
            .map(|e| e.1.execute(data).map(|r| (String::from(e.0), r)))
            .collect();
        self.custom_node
            .run(&self.output_var, parameters?, data, self.rest.as_ref())
    }
}

struct CompiledSink {
    node_id: String,
}

impl Interpreter for CompiledSink {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        Ok(ScenarioOutput(vec![SingleScenarioOutput {
            node_id: String::from(&self.node_id),
            variables: data.to_external_form(),
        }]))
    }
}

#[cfg(test)]
//These tests are a too high-level (at least some of them), but I had some technical
//problems splitting the code above (as I d)
mod tests {
    use crate::{
        interpreter::{
            compiler::Compiler,
            data::{ScenarioOutput, SingleScenarioOutput, VarContext, DEFAULT_INPUT_NAME},
        },
        scenariomodel::{
            Expression, MetaData, Node,
            Node::{Filter, Sink, Source, Variable},
            Scenario,
        },
    };
    use serde_json::json;
    use serde_json::Value;
    use std::collections::HashMap;

    fn js(expr: &str) -> Expression {
        Expression {
            language: String::from("javascript"),
            expression: String::from(expr),
        }
    }

    fn compile_invoke_to_output(node: Node, input: Value) -> ScenarioOutput {
        let scenario = Scenario {
            meta_data: MetaData {
                id: String::from(""),
            },
            nodes: vec![
                Source {
                    id: String::from("source"),
                },
                node,
                Sink {
                    id: String::from("sink"),
                },
            ],
        };
        let compiled_scenario = Compiler::default().compile(&scenario).unwrap();
        compiled_scenario
            .run(&VarContext::default_input(input))
            .unwrap()
    }

    #[test]
    fn test_variable() {
        let node = Variable {
            id: String::from("var"),
            var_name: String::from("new_var"),
            expression: js("12"),
        };
        let output = compile_invoke_to_output(node, json!(22));
        assert_eq!(
            output,
            ScenarioOutput(vec![SingleScenarioOutput {
                node_id: String::from("sink"),
                variables: HashMap::from([
                    (DEFAULT_INPUT_NAME.to_string(), json!(22)),
                    (String::from("new_var"), json!(12))
                ])
            }])
        )
    }

    #[test]
    fn test_filter() {
        let node = Filter {
            id: String::from("filter"),
            expression: js("input == 22"),
        };
        let output_true = compile_invoke_to_output(node, json!(22));
        assert_eq!(
            output_true,
            ScenarioOutput(vec![SingleScenarioOutput {
                node_id: String::from("sink"),
                variables: HashMap::from([(DEFAULT_INPUT_NAME.to_string(), json!(22))])
            }])
        );
        let node = Filter {
            id: String::from("filter"),
            expression: js("input == 22"),
        };
        let output_false = compile_invoke_to_output(node, json!(11));
        assert_eq!(output_false, ScenarioOutput(vec![]))
    }
}
