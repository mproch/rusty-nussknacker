
use std::{env, io::{self, BufRead}};
use rusty_nussknacker::{create_interpreter, invoke_interpreter, runtime::Interpreter};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
      panic!("Incorrect number of arguments: {}", args.len());
    }
    let interpreter = create_interpreter(&args[1]).unwrap();

    match args.len() {
      2 => {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
          invoke_on_line(interpreter.as_ref(), &line.unwrap())
        }
      },
      3 => {
        let input = &args[2];
        invoke_on_line(interpreter.as_ref(), input)

      }
      len => panic!("Incorrect number of arguments: {}", len)
    }
}

fn invoke_on_line(interpreter: &dyn Interpreter, input: &str) {
  let output = invoke_interpreter(interpreter, input).unwrap().0;
  println!("{}", serde_json::to_string(&output).unwrap())
}