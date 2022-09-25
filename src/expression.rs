use std::collections::HashMap;
use crate::interpreter::data::{VarContext, VarValue, ScenarioCompilationError, ScenarioRuntimeError, CompilationVarContext};
use crate::scenariomodel::Expression;
use super::javascriptexpression::JavaScriptParser;

pub trait Parser {
    fn parse(&self, expression: &str, var_context: &CompilationVarContext) -> Result<Box<dyn CompiledExpression>, ScenarioCompilationError>;
}

pub trait CompiledExpression {
    fn execute(&self, data: &VarContext) -> Result<VarValue, ScenarioRuntimeError>;
}


pub struct LanguageParser {
    parsers: HashMap<String, Box<dyn Parser>>
}

impl LanguageParser {
    pub fn parse(&self, expression: &Expression, var_context: &CompilationVarContext) -> Result<Box<dyn CompiledExpression>, ScenarioCompilationError> {
        let parser = self.parsers.get(&expression.language).ok_or_else(||ScenarioCompilationError(String::from("Unknown language")))?;
        parser.parse(&expression.expression, var_context)
    }

}

impl Default for LanguageParser {
    fn default() -> LanguageParser {
        let javascript: Box<dyn Parser> = Box::new(JavaScriptParser);
        LanguageParser { parsers: HashMap::from([(String::from("javascript"), javascript)]) }
    }
}


