mod commands;
mod config;
mod infrastructure;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::{
    check::CheckCommand, deploy::DeployCommand, generate::GenerateCommand, new::NewCommand,
};
use dotenv::dotenv;
use log::{Level, LevelFilter};
use thiserror::Error;

use crate::{
    commands::{prove::ProveCommand, verify::VerifyCommand},
    infrastructure::terminal::print_app_title,
};

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
    /// Create a new project
    New { target: String },
    /// Generate a verifier contract from a circuit
    Generate {
        #[arg(short, long)]
        package: Option<String>,
    },
    /// Check a verifier contract for stylus compatibility
    Check {
        #[arg(short, long)]
        package: Option<String>,
        #[arg(long)]
        rpc_url: Option<String>,
    },
    /// Deploy a verifier to the blockchain
    Deploy {
        #[arg(short, long)]
        package: Option<String>,
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        private_key: String,
        #[arg(long)]
        verifier_address: Option<String>,
        #[arg(long, default_value_t = false)]
        zk: bool,
    },
    /// Generate proof
    Prove {
        #[arg(short, long)]
        package: Option<String>,
        #[arg(long, default_value_t = false)]
        zk: bool,
    },
    /// Verify proof
    Verify {
        #[arg(long)]
        proof: Option<String>,
        #[arg(long)]
        public_input: Option<String>,
        #[arg(long)]
        vk: Option<String>,
        #[arg(long)]
        verifier_address: Option<String>,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long, default_value_t = false)]
        zk: bool,
    },
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum AppError {
    #[error("We can't find your contracts at {0}. Please run generate first.")]
    ContractsNotFound(PathBuf),
    #[error("Missing dependencies")]
    MissingDependencies(),
    #[error("Stylus error: {0}")]
    StylusError(String),
    #[error("Package not found")]
    PackageNotFound,
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Other error: {0}")]
    Other(&'static str),
    #[error("Compile error")]
    CompileError,
    #[error("Generate error")]
    GenerateError,
    #[error("Name is invalid: {0}")]
    InvalidName(String),
    #[error("Directory already exists: {0}")]
    DirectoryAlreadyExists(String),
}

pub(crate) struct AppContext {}

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

    // print app title
    clear_screen!();
    print_app_title();

    // run commands
    if let Err(e) = match args.cmd {
        Commands::New { target } => NewCommand::default().run(&ctx, &target).await,
        Commands::Generate { package } => GenerateCommand::default().run(&ctx, package).await,
        Commands::Check { package, rpc_url } => {
            CheckCommand::default().run(&ctx, package, rpc_url).await
        }
        Commands::Prove { package, zk } => ProveCommand::default().run(&ctx, package, zk).await,
        Commands::Verify {
            proof,
            public_input,
            vk,
            verifier_address,
            rpc_url,
            zk,
        } => {
            VerifyCommand::default()
                .run(&ctx, proof, public_input, vk, verifier_address, rpc_url, zk)
                .await
        }
        Commands::Deploy {
            package,
            rpc_url,
            private_key,
            verifier_address,
            zk,
        } => {
            DeployCommand::default()
                .run(&ctx, package, rpc_url, private_key, verifier_address, zk)
                .await
        }
    } {
        print_error!(" Error: {e} \n");
        std::process::exit(1);
    }
}
