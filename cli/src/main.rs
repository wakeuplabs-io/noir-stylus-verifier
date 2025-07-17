mod commands;
mod infrastructure;
mod config;

use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::init::InitCommand;
use dotenv::dotenv;
use log::{Level, LevelFilter};

use crate::infrastructure::console::terminal::print_error;

#[derive(Parser)]
#[clap(name = "nsv")]
#[clap(version = "0.1.0")]
#[clap(about = "Generate and deploy zk verifiers in stylus.", long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,

    /// Suppress logging output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Initialize a new project
    Init { target: String },
}

pub(crate) struct AppContext {
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args = Args::parse();

    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Off
    };

    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Off) // Turn off all logs by default
        .format(|f, record| {
            use std::io::Write;
            let target = record.target();
            let level = match record.level() {
                Level::Trace => "TRACE".red().to_string(),
                Level::Debug => "DEBUG".blue().to_string(),
                Level::Info => "INFO".green().to_string(),
                Level::Warn => "WARN".yellow().to_string(),
                Level::Error => "ERROR".red().to_string(),
            };
            writeln!(f, " {} {} > {}", level, target.bold(), record.args())
        })
        .filter_module("main", log_level)
        .init();

    let ctx = AppContext {};

    // run commands
    if let Err(e) = match args.cmd {
        Commands::Init { target } => InitCommand::new().run(&ctx, &target),
    } {
        print_error(&format!("\n\nError: {}\n\n", e));
        std::process::exit(1);
    }
}