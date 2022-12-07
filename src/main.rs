use rocket::State;
use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::path::Path;
use std::{path::PathBuf, process::exit};
use std::env;

#[macro_use]
extern crate rocket;

#[post("/", data = "<body>")]
fn invoke(body: &str, interpreter: &State<Box<dyn Interpreter>>) -> String {
    return match invoke_interpreter(interpreter.inner().as_ref(), body) {
        Ok(output) => serde_json::to_string(&output).unwrap(),
        Err(error) => format!("{}", error),
    };
}

#[get("/alive")]
fn alive() -> String {
    return String::from("OK");
}

#[get("/ready")]
fn ready() -> String {
    return String::from("OK");
}

#[launch]
fn rocket() -> _ {

    let name = env::var("SCENARIO_FILE").unwrap();
    let scenario_file = Path::new(&name);

    let interpreter = create_interpreter(scenario_file).unwrap_or_else(|err| {
        eprintln!("Failed to parse scenario: {err}");
        exit(1);
    });
    rocket::build()
        .manage(interpreter)
        .mount("/", routes![invoke, alive, ready])
}
