mod toml_conf;
mod commands;
mod utils;
mod zip_utils;

use std::{ process::ExitCode, thread };

use commands::{ handle_commands, CLI };

use clap::Parser;
use tokio::runtime;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() -> ExitCode {
    let args = CLI::parse();
    let rt = runtime::Builder::new_multi_thread().enable_time().build().unwrap();
    rt.block_on(handle_commands(args.cli));
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let child = thread::Builder::new().stack_size(STACK_SIZE).spawn(run).unwrap();
    child.join().unwrap()
}