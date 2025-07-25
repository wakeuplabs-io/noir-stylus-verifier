mod commands;
mod config;
mod infrastructure;

use crate::{
    commands::{prove::ProveCommand, verify::VerifyCommand},
    config::constants::DEFAULT_RPC_URL,
    infrastructure::terminal::print_app_title,
};
use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::{
    check::CheckCommand, deploy::DeployCommand, generate::GenerateCommand, new::NewCommand,
};
use dotenv::dotenv;
use log::{Level, LevelFilter};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser)]
#[clap(name = "nsv")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Generate and deploy verifiers in stylus from noir circuits.", long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,

    /// Show verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Create a new project
    New {
        #[arg(help = "Name of the project. This will also be the directory and package name.")]
        target: String,
    },
    ///  Generate a verifier contract from a noir circuit
    Generate {
        #[arg(short, long, help = "Package name containing the circuit")]
        package: Option<String>,
        #[arg(long, help = "Path to the bytecode to use for the proof generation")]
        bytecode_path: Option<String>,
        #[arg(
            long,
            help = "Path to the verification key to use for the proof generation"
        )]
        vk_path: Option<String>,
    },
    /// Check if the generated contract is compatible with Stylus, and how much it costs to deploy
    Check {
        #[arg(short, long, help = "Package name containing the circuit")]
        package: Option<String>,
        #[arg(
            long,
            help = "RPC URL to use for the check",
            default_value = DEFAULT_RPC_URL
        )]
        rpc_url: String,
    },
    /// Deploy the generated contract to the blockchain
    Deploy {
        #[arg(short, long, help = "Package name containing the circuit")]
        package: Option<String>,
        #[arg(long, help = "RPC URL to use for deployment")]
        rpc_url: String,
        #[arg(long, help = "Private key to sign the deployment transaction")]
        private_key: String,
        #[arg(
            long,
            help = "Address of the global verifier contract. Optional if using defaults (see `docs/deployments.md`)."
        )]
        verifier_address: Option<String>,
        #[arg(long, default_value_t = false, help = "Enable zk-flavored verifier")]
        zk: bool,
    },
    /// Generate a proof for a circuit. Useful for testing.
    Prove {
        #[arg(short, long, help = "Package name containing the circuit")]
        package: Option<String>,
        #[arg(
            long,
            default_value = "Prover.toml",
            help = "Name of the prover to use for the proof generation"
        )]
        prover_name: String,
        #[arg(
            long,
            default_value = "target",
            help = "Path where to output the proof and public inputs"
        )]
        output_path: String,
        #[arg(long, help = "Path to the witness to use for the proof generation")]
        witness_path: Option<String>,
        #[arg(long, help = "Path to the bytecode to use for the proof generation")]
        bytecode_path: Option<String>,
        #[arg(long, default_value_t = false, help = "Enable zk-flavored proof")]
        zk: bool,
    },
    /// Verify a proof for a circuit. Useful for testing.
    Verify {
        #[arg(
            long,
            default_value = "target/proof",
            help = "Path to the proof to verify"
        )]
        proof: String,
        #[arg(
            long,
            default_value = "target/public_inputs",
            help = "Path to the public input to verify"
        )]
        public_input: String,
        #[arg(
            long,
            default_value = "contracts/assets/vk",
            help = "Path to the verification key"
        )]
        vk: String,
        #[arg(
            long,
            help = "Address of the deployed verifier contract (defaults to local verifier if omitted)"
        )]
        verifier_address: Option<String>,
        #[arg(long, help = "RPC URL to use for verification")]
        rpc_url: Option<String>,
        #[arg(
            long,
            default_value_t = false,
            help = "Set if using a zk-flavored verifier and proof"
        )]
        zk: bool,
    },
}

#[derive(Error, Debug, PartialEq, Eq)]
pub(crate) enum AppError {
    #[error("We can't find your contracts at {0}. If they exist, try specifying the package with -p <package> or run nsv generate first")]
    ContractsNotFound(PathBuf),
    #[error("Missing dependencies: {0}")]
    MissingDependencies(String),
    #[error("Stylus error: {0}")]
    StylusError(String),
    #[error("Package not found")]
    PackageNotFound,
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
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
    #[error("No default verifier address found for chain")]
    NoDefaultVerifierAddress,
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
        Commands::Generate {
            package,
            bytecode_path,
            vk_path,
        } => {
            GenerateCommand::default()
                .run(&ctx, package, bytecode_path, vk_path)
                .await
        }
        Commands::Check { package, rpc_url } => {
            CheckCommand::default().run(&ctx, package, rpc_url).await
        }
        Commands::Prove {
            package,
            prover_name,
            output_path,
            witness_path,
            bytecode_path,
            zk,
        } => {
            ProveCommand::default()
                .run(
                    &ctx,
                    package,
                    prover_name,
                    output_path,
                    witness_path,
                    bytecode_path,
                    zk,
                )
                .await
        }
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
