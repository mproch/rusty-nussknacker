use clap::Parser;
use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::{
    io::{self, BufRead},
    path::PathBuf,
    process::exit,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub scenario_file: PathBuf,

    #[clap(value_parser)]
    pub input: Option<String>,
}

///This is just an example of how one can use the library. For more production-like usage,
///the interpreter would be run as a REST server, or a Kafka consumer.  
///As this is just an example usage of library, without too much logic, currently there are no tests...
fn main() {
    let args = Args::parse();

    let interpreter = create_interpreter(args.scenario_file.as_path()).unwrap_or_else(|err| {
        eprintln!("Failed to parse scenario: {err}");
        exit(1);
    });

    match args.input {
        None => {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                invoke_on_line(interpreter.as_ref(), &line.unwrap())
            }
        }
        Some(input) => invoke_on_line(interpreter.as_ref(), &input),
    }
}

fn invoke_on_line(interpreter: &dyn Interpreter, input: &str) {
    match invoke_interpreter(interpreter, input) {
        Ok(output) => println!("{}", serde_json::to_string(&output).unwrap()),
        Err(error) => eprintln!("{}", error),
    }
}
