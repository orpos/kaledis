mod android;
mod commands;
mod dalbit;
mod home_manager;
mod live_var_lsp;
mod toml_conf;
mod utils;
mod zip_utils;

use std::{env, io::Write, process::ExitCode, thread};

use colored::Colorize;
use commands::{CLI, handle_commands};

use clap::Parser;
use fs_err::File;
use tokio::runtime;

use crate::toml_conf::{KaledisConfig, LoveConfig};

const STACK_SIZE: usize = 4 * 1024 * 1024 * 1024;

fn print_banner() {
    println!(
        "{}",
        "░  ░░░░  ░░░      ░░░  ░░░░░░░░        ░░       ░░░        ░░░      ░░".purple()
    );
    println!(
        "{}",
        "▒  ▒▒▒  ▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒".purple()
    );
    println!(
        "{}",
        "▓     ▓▓▓▓▓  ▓▓▓▓  ▓▓  ▓▓▓▓▓▓▓▓      ▓▓▓▓  ▓▓▓▓  ▓▓▓▓▓  ▓▓▓▓▓▓      ▓▓".cyan()
    );
    println!(
        "{}",
        "█  ███  ███        ██  ████████  ████████  ████  █████  ███████████  █".cyan()
    );
    println!(
        "{}",
        "█  ████  ██  ████  ██        ██        ██       ███        ███      ██".cyan()
    );
    println!("");
}

fn run() -> ExitCode {
    print_banner();
    // I use this to generate the schema
    {
        let schema = schemars::schema_for!(KaledisConfig);
        let schema2 = schemars::schema_for!(LoveConfig);
        File::create("kaledis.schema.json")
            .unwrap()
            .write_all(serde_json::to_string(&schema).unwrap().as_bytes())
            .unwrap();
        File::create("love.schema.json")
            .unwrap()
            .write_all(serde_json::to_string(&schema2).unwrap().as_bytes())
            .unwrap();
    };
    let args = CLI::parse();
    let rt = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    rt.block_on(handle_commands(args.cli));
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();
    child.join().unwrap_or(ExitCode::FAILURE)
}
