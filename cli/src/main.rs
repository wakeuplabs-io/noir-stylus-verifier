mod commands;
mod config;
mod infrastructure;

use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::{
    check::CheckCommand, deploy::DeployCommand, generate::GenerateCommand, new::NewCommand,
};
use dotenv::dotenv;
use log::{Level, LevelFilter};

use crate::infrastructure::console::terminal::print_app_title;

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
    /// Generate a verifier
    Generate {
        #[arg(short, long)]
        circuit: Option<String>,
    },
    /// Check a verifier contract
    Check {
        #[arg(short, long)]
        circuit: Option<String>,
        #[arg(short, long)]
        rpc_url: Option<String>,
    },
    /// Deploy a verifier to the blockchain
    Deploy {
        #[arg(short, long)]
        circuit: Option<String>,
        #[arg(short, long)]
        rpc_url: String,
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        verifier_address: Option<String>,
        #[arg(short, long, default_value_t = false)]
        zk_flavor: bool,
    },
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
        Commands::New { target } => NewCommand::new().run(&ctx, &target).await,
        Commands::Generate { circuit } => GenerateCommand::new().run(&ctx, circuit).await,
        Commands::Check { circuit, rpc_url } => {
            CheckCommand::default().run(&ctx, circuit, rpc_url).await
        }
        Commands::Deploy {
            circuit,
            rpc_url,
            private_key,
            verifier_address,
            zk_flavor,
        } => {
            DeployCommand::default()
                .run(
                    &ctx,
                    circuit,
                    rpc_url,
                    private_key,
                    verifier_address,
                    zk_flavor,
                )
                .await
        }
    } {
        print_error!(" Error: {e} \n");
        std::process::exit(1);
    }
}
