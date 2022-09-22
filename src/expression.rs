use crate::runtime::data::{InputData, VarValue, ScenarioError::{*, self}, VarContext};

pub mod parse;
use crate::data::jsonmodel::Expression;

pub type Parser = fn (Expression, VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioError>;

pub trait CompiledExpression {
    fn execute(&self, data: &InputData) -> Result<VarValue, ScenarioError>;
}
