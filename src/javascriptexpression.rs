use super::expression::{CompiledExpression, Parser};
use crate::interpreter::data::{
    CompilationVarContext, ScenarioCompilationError, ScenarioRuntimeError, VarContext, VarValue,
};
use js_sandbox::Script;
use serde_json::Value;

pub struct JavaScriptParser;

impl Parser for JavaScriptParser {
    fn parse(
        &self,
        expression: &str,
        var_context: &CompilationVarContext,
    ) -> Result<Box<dyn CompiledExpression>, ScenarioCompilationError> {
        let keys = var_context
            .0
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        let expanded = format!(
            r#"function run (argMap) {{
            const {{ {} }} = argMap
            return ({})
        }}"#,
            keys, expression
        );
        let expr = JavascriptExpression {
            transformed: expanded,
        };
        Ok(Box::new(expr))
    }
}

struct JavascriptExpression {
    transformed: String,
}

impl CompiledExpression for JavascriptExpression {
    fn execute(&self, input_data: &VarContext) -> Result<VarValue, ScenarioRuntimeError> {
        let mut expression = Script::from_string(&self.transformed)
            .map_err(|err| ScenarioRuntimeError(err.to_string()))?;
        let converted = serde_json::to_value(&input_data.to_external_form())
            .map_err(|err| ScenarioRuntimeError(err.to_string()))?;
        expression
            .call::<Value, Value>("run", &converted)
            .map_err(|err| ScenarioRuntimeError(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expression::Parser,
        interpreter::data::{CompilationVarContext, VarContext},
        javascriptexpression::JavaScriptParser,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_simple_expression() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser.parse("10 + 5", &CompilationVarContext(HashMap::new()))?;
        let res = expr.execute(&VarContext(HashMap::from([])))?;
        assert_eq!(res, json!(15));
        Ok(())
    }

    #[test]
    fn test_expression_with_variable() -> Result<(), Box<dyn std::error::Error>> {
        let expr = JavaScriptParser.parse(
            "intval + 5",
            &CompilationVarContext(HashMap::from([(String::from("intval"), ())])),
        )?;
        let res = expr.execute(&VarContext(HashMap::from([(
            String::from("intval"),
            Rc::new(serde_json::to_value(10)?),
        )])))?;
        assert_eq!(res, serde_json::to_value(15)?);
        Ok(())
    }
}
