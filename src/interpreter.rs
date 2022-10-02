use std::collections::HashMap;

use self::data::{
    ScenarioCompilationError, ScenarioOutput, ScenarioRuntimeError, VarContext, VarValue,
};

pub mod compiler;
pub mod data;

///This is the main API of the rusty-nussknacker library. It represents 'compiled' scenario,
///which can transform input - VarContext into ScenarioOutput
pub trait Interpreter {
    fn run(&self, data: &VarContext) -> Result<ScenarioOutput, ScenarioRuntimeError>;
}

pub type CompilationResult = Result<Box<dyn Interpreter>, ScenarioCompilationError>;

///This is the API of different kinds of components that may be plugged into the library.
///Given input, evaluated parameters and continuation of rest of the scenario (next_part parameter),
///implementations of the trait compute the output.
///Note, that the API allows next_part to be invoked 0..many times, which allows to implement different types
///of components, from filter to for-each types.
///Sample implementation of for-each is provided in customnodes module.
pub trait CustomNodeImpl {
    fn run(
        &self,
        output_var: &str,
        parameters: &HashMap<String, VarValue>,
        input: &VarContext,
        next_part: &dyn Interpreter,
    ) -> Result<ScenarioOutput, ScenarioRuntimeError>;
}
