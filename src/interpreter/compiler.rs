use super::{
    data::{CompilationVarContext, ScenarioCompilationError},
    CompilationResult, CustomNodeImpl, Interpreter,
};
use crate::{
    customnodes::ForEach,
    expression::LanguageParser,
    scenariomodel::{Node, Node::*, NodeId, Scenario},
};
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
            Some(Source { id }) => self.compile_next(id, &iter[1..], &initial_input),
            Some(other) => Err(ScenarioCompilationError::FirstNodeNotSource(
                other.id().clone(),
            )),
            None => Err(ScenarioCompilationError::EmptyScenario()),
        };
    }

    fn compile_next(
        &self,
        node_id: &NodeId,
        iter: &[Node],
        var_names: &CompilationVarContext,
    ) -> CompilationResult {
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
            Filter { id: _, expression } => filter::compile(ctx, expression),
            Variable {
                id: _,
                var_name,
                expression,
            } => variable::compile(ctx, var_name, expression),
            Switch { id: _, nexts } => switch::compile(ctx, nexts),
            Split { id: _, nexts } => split::compile(ctx, nexts),
            Sink { id } => sink::compile(ctx, id),
            CustomNode {
                id,
                output_var,
                node_type,
                parameters,
            } => customnode::compile(
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
        self.custom_nodes.get(node_type).ok_or_else(|| {
            ScenarioCompilationError::UnknownCustomNode {
                node_id: node_id.clone(),
                node_type: node_type.to_string(),
            }
        })
    }
}

mod customnode;
mod filter;
mod sink;
mod split;
mod switch;
mod variable;

//I was hoping that if child modules are not public, I will be able to make this struct private, but somehow I can't...
pub struct CompilationContext<'a> {
    parser: &'a LanguageParser,
    compiler: &'a dyn Fn(&[Node], &CompilationVarContext) -> CompilationResult,
    var_names: &'a CompilationVarContext,
    rest: &'a [Node],
    node_id: &'a NodeId,
}

impl CompilationContext<'_> {
    fn assert_end(&self, value: Box<dyn Interpreter>) -> CompilationResult {
        if self.rest.is_empty() {
            Ok(value)
        } else {
            Err(ScenarioCompilationError::NodesAfterEndingNode {
                node_id: self.node_id.clone(),
                unexpected_nodes: self.rest.to_vec(),
            })
        }
    }
}

#[cfg(test)]
//These tests are a bit too high-level (at least some of them), but I've figured out how to split compiler only at last time
mod tests {
    use crate::{
        interpreter::{
            compiler::Compiler,
            data::{ScenarioOutput, SingleScenarioOutput, VarContext, DEFAULT_INPUT_NAME},
        },
        scenariomodel::{
            Expression, MetaData,
            Node::{Filter, Sink, Source, Variable},
            Scenario,
        },
    };
    use crate::{
        interpreter::{data::CompilationVarContext, CompilationResult},
        scenariomodel::{Node, NodeId},
    };
    use serde_json::json;
    use serde_json::Value;
    use std::collections::HashMap;

    pub fn compile_node(node: Node, rest: &[Node]) -> CompilationResult {
        let var_ctx = CompilationVarContext::default();
        Compiler::default().compile_next_node(&node, rest, &var_ctx)
    }

    pub fn js(value: &str) -> Expression {
        Expression {
            language: String::from("javascript"),
            expression: String::from(value),
        }
    }

    pub fn sink(id: &NodeId) -> Vec<Node> {
        vec![{ Node::Sink { id: id.clone() } }]
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
