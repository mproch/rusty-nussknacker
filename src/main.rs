
use std::env;
use rusty_nussknacker::interpret_scenario;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
      panic!("Incorrect number of arguments: {}", args.len());
    }
    let file_name = args[1].clone();
    let input = args[2].clone();
    let output = interpret_scenario(&file_name, &input).unwrap().0;
    
    println!("{}", serde_json::to_string(&output[0]).unwrap());
}