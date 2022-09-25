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
    fn test_simple_expression() {
        //not quite sure how to import for test properly, without warning

        let expr = JavaScriptParser
            .parse("10 + 5", &CompilationVarContext(HashMap::new()))
            .unwrap();
        let res = expr.execute(&VarContext(HashMap::from([])));
        assert_eq!(res.unwrap(), json!(15));
    }

    #[test]
    fn test_expression_with_variable() {
        let expr = JavaScriptParser
            .parse(
                "ala + 5",
                &CompilationVarContext(HashMap::from([(String::from("ala"), ())])),
            )
            .unwrap();
        let res = expr.execute(&VarContext(HashMap::from([(
            String::from("ala"),
            Rc::new(serde_json::to_value(10).unwrap()),
        )])));
        assert_eq!(res.unwrap(), serde_json::to_value(15).unwrap());
    }
}
