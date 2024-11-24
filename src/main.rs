mod toml_conf;
mod commands;
mod utils;
mod zip_utils;

use std::{ env::current_exe, fs::{ self, remove_file }, path::PathBuf, process::ExitCode, thread };

use commands::{ handle_commands, CLI };

use clap::Parser;
use tokio::runtime;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() -> ExitCode {

    if
        std::env
            ::args()
            .nth(1)
            .map(|x| x.starts_with("__new__"))
            .unwrap_or(false)
    {
        let target = std::env::args().nth(2).unwrap();
        let ddd = PathBuf::from(target);
        remove_file(&ddd).unwrap();
        fs::copy(current_exe().unwrap(), ddd).unwrap();

        return ExitCode::SUCCESS;
    }
    let args = CLI::parse();
    let rt = runtime::Builder::new_multi_thread().enable_time().enable_io().build().unwrap();
    rt.block_on(handle_commands(args.cli));
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let child = thread::Builder::new().stack_size(STACK_SIZE).spawn(run).unwrap();
    child.join().unwrap()
}