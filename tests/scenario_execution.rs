use std::error;
use std::path::PathBuf;

use rusty_nussknacker::create_interpreter;
use rusty_nussknacker::interpreter::data::VarContext;
use rusty_nussknacker::scenariomodel::NodeId;
use serde_json::json;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[test]
fn test_scenario_with_custom_node() -> Result<()> {
    let interpreter = create_interpreter(scenario("with_custom.json").as_path())?;

    let input = VarContext::default_input(json!(""));
    let output = interpreter.run(&input)?;
    assert_eq!(
        output.var_in_sink(&NodeId::new("sink"), "each"),
        vec![Some(&json!("a")), Some(&json!("b")), Some(&json!("c"))]
    );
    Ok(())
}

#[test]
fn test_scenario_with_split() -> Result<()> {
    let interpreter = create_interpreter(scenario("with_split.json").as_path())?;

    let input = VarContext::default_input(json!(4));
    let output = interpreter.run(&input)?;
    assert_eq!(
        output.var_in_sink(&NodeId::new("sink1"), "additional"),
        vec![Some(&json!(true))]
    );
    assert_eq!(
        output.var_in_sink(&NodeId::new("sink2"), "additional"),
        vec![Some(&json!(true))]
    );
    Ok(())
}

fn scenario(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/scenarios");
    d.push(name);
    d
}
