use std::io::prelude::*;
use std::{error::Error, fs::File};

use deno_core::{JsRuntime, RuntimeOptions};

fn main() {
    create_snapshot().unwrap();
}

fn create_snapshot() -> Result<(), Box<dyn Error>> {
    let options = RuntimeOptions {
        will_snapshot: true,
        ..Default::default()
    };
    let mut runtime = JsRuntime::new(options);

    let mut snap = File::create("snapshots/query_runtime.snap")?;
    snap.write_all(&runtime.snapshot())?;

    Ok(())
}
