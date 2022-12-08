use rocket::State;
use rusty_nussknacker::{create_interpreter, interpreter::Interpreter, invoke_interpreter};
use std::env;
use std::path::Path;
use std::process::exit;

#[macro_use]
extern crate rocket;

#[post("/", data = "<body>")]
async fn invoke(body: &str, interpreter: &State<Box<dyn Interpreter>>) -> String {
    match invoke_interpreter(interpreter.inner().as_ref(), body).await {
        Ok(output) => serde_json::to_string(&output).unwrap(),
        Err(error) => format!("{}", error),
    }
}

#[get("/alive")]
fn alive() -> String {
    String::from("OK")
}

#[get("/ready")]
fn ready() -> String {
    String::from("OK")
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
