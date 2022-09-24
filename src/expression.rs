use std::collections::HashMap;

use crate::runtime::data::{InputData, VarValue, ScenarioError, VarContext};
use crate::runtime::data::ScenarioError::ScenarioCompilationError;

pub mod parse;
use crate::data::jsonmodel::Expression;

use self::parse::JavaScriptParser;

pub trait Parser {
    fn parse(&self, expression: &str, var_context: &VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioError>;
}

pub trait CompiledExpression {
    fn execute(&self, data: &InputData) -> Result<VarValue, ScenarioError>;
}


pub struct LanguageParser {
    parsers: HashMap<String, Box<dyn Parser>>
}

impl LanguageParser {
    pub fn parse(&self, expression: &Expression, var_context: &VarContext) -> Result<Box<dyn CompiledExpression>, ScenarioError> {
        let parser = self.parsers.get(&expression.language).ok_or(ScenarioCompilationError(String::from("Unknown language")))?;
        return parser.parse(&expression.expression, var_context)
    }

}

impl Default for LanguageParser {
    fn default() -> LanguageParser {
        let javascript: Box<dyn Parser> = Box::new(JavaScriptParser);
        return LanguageParser { parsers: HashMap::from([(String::from("javascript"), javascript)]) }
    }
}


