use std::fs;
use std::io;
use super::jsonmodel::Scenario;

pub fn parse(scenario_filename: &str) -> Result<Scenario, io::Error>  {
    let scenario_json = fs::read_to_string(scenario_filename)?;
    let scenario: Scenario = serde_json::from_str(&scenario_json)?;
    Ok(scenario)
}