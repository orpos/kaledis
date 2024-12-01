use std::{process::ExitCode, thread};

mod cli;

use clap::Parser;
use cli::Dal;
pub use dal_core;
use tokio::runtime;
use anstyle::{AnsiColor, Color, Style};
use env_logger::Builder;
use log::Level;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() -> ExitCode {
    let rt = runtime::Builder::new_multi_thread().build().unwrap();

    let dal = Dal::parse();

    let filter = dal.get_log_level_filter();

    formatted_logger().filter_module("dal", filter).init();

    match rt.block_on(dal.run()) {
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

fn formatted_logger() -> Builder {
    let mut builder = Builder::from_default_env();
    builder.format(|f, record| {
        use std::io::Write;

        let level = record.level();
        let (style, text) = colored_level(level);

        writeln!(f, " {style}{text}{style:#} > {}", record.args())
    });
    builder
}

fn colored_level(level: Level) -> (Style, &'static str) {
    let (color, text) = match level {
        Level::Trace => (AnsiColor::Magenta, "TRACE"),
        Level::Debug => (AnsiColor::Blue, "DEBUG"),
        Level::Info => (AnsiColor::Green, "INFO"),
        Level::Warn => (AnsiColor::Yellow, "WARN"),
        Level::Error => (AnsiColor::Red, "ERROR"),
    };
    (Style::new().fg_color(Some(Color::Ansi(color))), text)
}
