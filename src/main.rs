use clap::Parser;
use rocket::State;
use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::{path::PathBuf, process::exit};

#[macro_use]
extern crate rocket;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub scenario_file: PathBuf,

    #[clap(value_parser)]
    pub input: Option<String>,
}

#[post("/", data = "<body>")]
fn invoke(body: &str, interpreter: &State<Box<dyn Interpreter>>) -> String {
    return match invoke_interpreter(interpreter.inner().as_ref(), body) {
        Ok(output) => serde_json::to_string(&output).unwrap(),
        Err(error) => format!("{}", error),
    };
}

#[launch]
fn rocket() -> _ {
    let args = Args::parse();

    let interpreter = create_interpreter(args.scenario_file.as_path()).unwrap_or_else(|err| {
        eprintln!("Failed to parse scenario: {err}");
        exit(1);
    });
    rocket::build()
        .manage(interpreter)
        .mount("/", routes![invoke])
}
