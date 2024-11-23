use std::{process::ExitCode, thread};

mod cli;

use cli::Dal;
pub use dal_core;
use tokio::runtime;

const STACK_SIZE: usize = 4 * 1024 * 1024;

// #[tokio::main(flavor = "multi_thread")]
fn run() -> ExitCode {
    let rt = runtime::Builder::new_multi_thread().build().unwrap();
    match rt.block_on(Dal::new().run()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    child.join().unwrap()
}
