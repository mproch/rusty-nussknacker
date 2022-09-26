use std::error;
use std::path::PathBuf;

use rusty_nussknacker::create_interpreter;
use rusty_nussknacker::interpreter::data::VarContext;
use serde_json::json;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[test]
fn test_scenario_running() -> Result<()> {
    let interpreter = create_interpreter(scenario("with_custom.json").as_path())?;

    let input = VarContext::default_input(json!(""));
    let output = interpreter.run(&input)?;

    Ok(())
}

fn scenario(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/scenarios");
    d.push(name);
    d
}
