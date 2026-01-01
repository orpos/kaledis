mod cli_utils;
mod commands;
mod toml_conf;
mod utils;
mod zip_utils;
mod dalbit;

use std::{process::ExitCode, thread};

use colored::Colorize;
use commands::{handle_commands, CLI};

use clap::Parser;
use tokio::runtime;

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
    return child.join().unwrap_or(ExitCode::FAILURE);
}
