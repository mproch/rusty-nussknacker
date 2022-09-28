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
    scenariomodel::{Case, Expression, Node, Node::*, NodeId, Parameter, Scenario},
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

impl Compiler {
    pub fn compile(&self, scenario: &Scenario) -> CompilationResult {
        let iter = &scenario.nodes;
        let initial_input = CompilationVarContext::default();
        return match iter.first() {
            Some(Source { id }) => self.compile_next(id,&iter[1..], &initial_input),
            Some(other) => Err(ScenarioCompilationError::FirstNodeNotSource(other.id().clone())),
            None => Err(ScenarioCompilationError::EmptyScenario()),
        };
    }

    fn compile_next(&self, node_id: &NodeId, iter: &[Node], var_names: &CompilationVarContext) -> CompilationResult {
        match iter.first() {
            Some(first) => self.compile_next_node(first, &iter[1..], var_names),
            None => Err(ScenarioCompilationError::InvalidEnd(node_id.clone())),
        }
    }

    fn compile_next_node(
        &self,
        head: &Node,
        rest: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
        let ctx = CompilationContext {
            parser: &self.parser,
            var_names,
            rest,
            node_id: head.id(),
            compiler: &|nds, ctx| self.compile_next(head.id(), nds, ctx),
        };
        match head {
            Filter { id: _, expression } => CompiledFilter::compile(ctx, expression),
            Variable {
                id: _,
                var_name,
                expression,
            } => CompiledVariable::compile(ctx, var_name, expression),
            Switch { id: _, nexts } => CompiledSwitch::compile(ctx, nexts),
            Split { id: _, nexts } => CompiledSplit::compile(ctx, nexts),
            Sink { id } => CompiledSink::compile(ctx, id),
            CustomNode {
                id,
                output_var,
                node_type,
                parameters,
            } => CompiledCustomNode::compile(
                ctx,
                output_var,
                parameters,
                self.custom_node(id, node_type)?,
            ),
            other => Err(ScenarioCompilationError::UnknownNode(other.id().clone())),
        }
    }

    fn custom_node(
        &self,
        node_id: &NodeId, 
        node_type: &str,
    ) -> Result<&Rc<dyn CustomNodeImpl>, ScenarioCompilationError> {
        self.custom_nodes
            .get(node_type)
            .ok_or_else(|| ScenarioCompilationError::UnknownCustomNode { node_id: node_id.clone(), node_type: node_type.to_string() } )
    }
}

struct CompilationContext<'a> {
    parser: &'a LanguageParser,
    compiler: &'a dyn Fn(&[Node], &CompilationVarContext) -> CompilationResult,
    var_names: &'a CompilationVarContext,
    rest: &'a [Node],
    node_id: &'a NodeId
}

impl CompilationContext<'_> {
    fn assert_end(&self, value: Box<dyn Interpreter>) -> CompilationResult {
        if self.rest.is_empty() {
            Ok(value)
        } else {
            Err(ScenarioCompilationError::NodesAfterEndingNode { node_id: self.node_id.clone(), unexpected_nodes: self.rest.to_vec() })
        }
    }
}

struct CompiledVariable {
    rest: Box<dyn Interpreter>,
    expression: Box<dyn CompiledExpression>,
    var_name: String,
}

impl CompiledVariable {
    fn compile(
        ctx: CompilationContext,
        var_name: &str,
        raw_expression: &Expression,
    ) -> Result<Box<dyn Interpreter>, ScenarioCompilationError> {
        let expression = ctx.parser.parse(ctx.node_id, raw_expression, ctx.var_names)?;
        let rest = (ctx.compiler)(ctx.rest, &ctx.var_names.with_var(ctx.node_id, var_name)?)?;
        Ok(Box::new(CompiledVariable {
            rest,
            expression,
            var_name: String::from(var_name),
        }))
    }
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

impl CompiledSplit {
    fn compile(ctx: CompilationContext, nexts: &[Vec<Node>]) -> CompilationResult {
        let compiled: Result<Vec<Box<dyn Interpreter>>, ScenarioCompilationError> = nexts
            .iter()
            .map(|n| (ctx.compiler)(&n[..], ctx.var_names))
            .collect();
        ctx.assert_end(Box::new(CompiledSplit { nexts: compiled? }))
    }
}

impl Interpreter for CompiledSplit {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        let output_result: Result<Vec<ScenarioOutput>, ScenarioRuntimeError> =
            self.nexts.iter().map(|one| one.run(data)).collect();
        output_result.map(ScenarioOutput::flatten)
    }
}

struct CompiledSwitch {
    nexts: Vec<CompiledCase>,
}

impl CompiledSwitch {
    fn compile(ctx: CompilationContext, nexts: &[Case]) -> CompilationResult {
        let parse_case = |case: &Case| {
            let rest = (ctx.compiler)(&case.nodes[..], ctx.var_names)?;
            let expression = ctx.parser.parse(ctx.node_id, &case.expression, ctx.var_names)?;
            Ok(CompiledCase { rest, expression })
        };
        let compiled: Result<Vec<CompiledCase>, ScenarioCompilationError> =
            nexts.iter().map(parse_case).collect();
        ctx.assert_end(Box::new(CompiledSwitch { nexts: compiled? }))
    }
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

impl CompiledFilter {
    fn compile(ctx: CompilationContext, expression: &Expression) -> CompilationResult {
        let rest = (ctx.compiler)(ctx.rest, ctx.var_names)?;
        let expression = ctx.parser.parse(ctx.node_id, expression, ctx.var_names)?;
        let res = CompiledFilter { rest, expression };
        Ok(Box::new(res))
    }
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

impl CompiledCustomNode {
    fn compile(
        ctx: CompilationContext,
        output_var: &str,
        parameters: &[Parameter],
        implementation: &Rc<dyn CustomNodeImpl>,
    ) -> CompilationResult {
        let next_part = (ctx.compiler)(ctx.rest, &ctx.var_names.with_var(ctx.node_id, output_var)?)?;
        let compiled_parameters: Result<
            HashMap<String, Box<dyn CompiledExpression>>,
            ScenarioCompilationError,
        > = parameters
            .iter()
            .map(|p| CompiledCustomNode::compile_parameter(&ctx, p))
            .collect();
        Ok(Box::new(CompiledCustomNode {
            rest: next_part,
            output_var: String::from(output_var),
            params: compiled_parameters?,
            custom_node: implementation.clone(),
        }))
    }

    fn compile_parameter(
        ctx: &CompilationContext,
        parameter: &Parameter,
    ) -> Result<(String, Box<dyn CompiledExpression>), ScenarioCompilationError> {
        let compiled_expression = ctx.parser.parse(ctx.node_id, &parameter.expression, ctx.var_names)?;
        Ok((parameter.name.clone(), compiled_expression))
    }
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
    node_id: NodeId,
}

impl CompiledSink {
    fn compile(ctx: CompilationContext, sink_id: &NodeId) -> CompilationResult {
        ctx.assert_end(Box::new(CompiledSink {
            node_id: sink_id.clone(),
        }))
    }
}

impl Interpreter for CompiledSink {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError> {
        Ok(ScenarioOutput(vec![SingleScenarioOutput {
            node_id: self.node_id.clone(),
            variables: data.to_external_form(),
        }]))
    }
}

#[cfg(test)]
//These tests are a too high-level (at least some of them), but I had some technical
//problems splitting the code above, so some of them will have to be added later...
mod tests {
    use crate::{
        interpreter::{
            compiler::Compiler,
            data::{ScenarioOutput, SingleScenarioOutput, VarContext, DEFAULT_INPUT_NAME},
        },
        scenariomodel::{
            Expression, MetaData, Node,
            Node::{Filter, Sink, Source, Variable},
            NodeId, Scenario,
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
                    id: NodeId::new("source"),
                },
                node,
                Sink {
                    id: NodeId::new("sink"),
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
            id: NodeId::new("var"),
            var_name: String::from("new_var"),
            expression: js("12"),
        };
        let output = compile_invoke_to_output(node, json!(22));
        assert_eq!(
            output,
            ScenarioOutput(vec![SingleScenarioOutput {
                node_id: NodeId::new("sink"),
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
            id: NodeId::new("filter"),
            expression: js("input == 22"),
        };
        let output_true = compile_invoke_to_output(node, json!(22));
        assert_eq!(
            output_true,
            ScenarioOutput(vec![SingleScenarioOutput {
                node_id: NodeId::new("sink"),
                variables: HashMap::from([(DEFAULT_INPUT_NAME.to_string(), json!(22))])
            }])
        );
        let node = Filter {
            id: NodeId::new("filter"),
            expression: js("input == 22"),
        };
        let output_false = compile_invoke_to_output(node, json!(11));
        assert_eq!(output_false, ScenarioOutput(vec![]))
    }

}
