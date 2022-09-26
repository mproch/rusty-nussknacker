use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::{
    env,
    io::{self, BufRead},
    path::Path,
};

///This is just an example of how one can use the library. For more production-like usage,
///the interpreter would be run as a REST server, or a Kafka consumer.  
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Incorrect number of arguments: {}", args.len());
    }
    let interpreter = create_interpreter(Path::new(&args[1])).unwrap();

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
        len => panic!("Incorrect number of arguments: {}", len),
    }
}

fn invoke_on_line(interpreter: &dyn Interpreter, input: &str) {
    let output = invoke_interpreter(interpreter, input).unwrap().0;
    println!("{}", serde_json::to_string(&output).unwrap())
}
