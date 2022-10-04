use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::{
    env,
    io::{self, BufRead},
    path::Path,
    process::exit,
};

///This is just an example of how one can use the library. For more production-like usage,
///the interpreter would be run as a REST server, or a Kafka consumer.  
///As this is just an example usage of library, without too much logic, currently there are no tests...
fn main() {
    let args: Vec<String> = env::args().collect();
    let print_usage_and_quit = || {
        eprintln!("Incorrect number of arguments: {}, usage: rusty-nussknacker scenario_file [scenario_input]. 
    If input is not provided, it will be read from stdin", args.len()-1);
        exit(1);
    };

    if args.len() < 2 {
        print_usage_and_quit();
    }
    let interpreter = create_interpreter(Path::new(&args[1])).unwrap_or_else(|err| {
        eprintln!("Failed to parse scenario: {err}");
        exit(1);
    });

    match args.len() {
        2 => {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                invoke_on_line(interpreter.as_ref(), &line.unwrap())
            }
        }
        3 => {
            let input = &args[2];
            invoke_on_line(interpreter.as_ref(), input)
        }
        _ => print_usage_and_quit(),
    }
}

fn invoke_on_line(interpreter: &dyn Interpreter, input: &str) {
    match invoke_interpreter(interpreter, input) {
        Ok(output) => println!("{}", serde_json::to_string(&output).unwrap()),
        Err(error) => eprintln!("{}", error),
    }
}
