use super::javascriptexpression::JavaScriptParser;
use crate::interpreter::data::{
    CompilationVarContext, ScenarioCompilationError, ScenarioCompilationError::UnknownLanguage,
    ScenarioRuntimeError, VarContext, VarValue,
};
use crate::scenariomodel::Expression;
use std::collections::HashMap;

pub trait ParseError: std::error::Error {}

pub trait Parser {
    fn parse(
        &self,
        expression: &str,
        var_context: &CompilationVarContext,
    ) -> Result<Box<dyn CompiledExpression>, Box<dyn ParseError>>;
}

pub trait CompiledExpression {
    fn execute(&self, data: &VarContext) -> Result<VarValue, ScenarioRuntimeError>;
}

pub struct LanguageParser {
    parsers: HashMap<String, Box<dyn Parser>>,
}

impl LanguageParser {
    pub fn parse(
        &self,
        expression: &Expression,
        var_context: &CompilationVarContext,
    ) -> Result<Box<dyn CompiledExpression>, ScenarioCompilationError> {
        let parser = self
            .parsers
            .get(&expression.language)
            .ok_or_else(|| UnknownLanguage(expression.language.to_string()))?;
        parser
            .parse(&expression.expression, var_context)
            .map_err(|err| ScenarioCompilationError::ParseError(err))
    }
}

impl Default for LanguageParser {
    fn default() -> LanguageParser {
        let javascript: Box<dyn Parser> = Box::new(JavaScriptParser);
        LanguageParser {
            parsers: HashMap::from([(String::from("javascript"), javascript)]),
        }
    }
}
