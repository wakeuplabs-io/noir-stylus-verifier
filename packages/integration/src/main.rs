//! Basic tests for Stylus programs. These assume that a devnet is already
//! running locally.

use crate::{
    cli::TestVerbosity,
    errors::ScriptError,
    types::{IntegrationTest, IntegrationTestFn, StylusContract},
    utils::{build_stylus_contract, deploy_stylus_contract, setup_client, LocalWalletHttpClient},
};
use abis::{PrecompileTestContract::PrecompileTestContractInstance, VerifierContract::VerifierContractInstance};
use alloy::{network::Ethereum, primitives::Address, providers::DynProvider};
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use std::io::{stdout, Write};
use std::process::exit;

mod abis;
mod assertions;
mod cli;
mod constants;
mod errors;
mod tests;
mod types;
mod utils;

/// An instance of the verifier contract
pub type VerifierTestInstance = VerifierContractInstance<(), DynProvider, Ethereum>;
/// An instance of the precompile test contract
pub type PrecompileTestInstance = PrecompileTestContractInstance<(), DynProvider, Ethereum>;

/// The context provided to each integration test
///
/// Allows for dependency and argument injection as well as convenient helpers
/// for setting up tests
#[derive(Clone)]
pub struct TestContext {
    /// The RPC client
    pub client: LocalWalletHttpClient,
    /// The address of the precompiles testing contract
    pub precompiles_contract_address: Address,
    /// The address of the verifier contract
    pub verifier_contract_address: Address,
}

impl TestContext {
    async fn try_new(value: Cli) -> Result<Self, ScriptError> {
        let client = setup_client(&value.priv_key, &value.rpc_url).await.unwrap();

        // build contracts
        build_stylus_contract(&StylusContract::PrecompileTestContract).unwrap();
        build_stylus_contract(&StylusContract::SumcheckVerifier).unwrap();
        build_stylus_contract(&StylusContract::ShpleminiVerifier).unwrap();
        build_stylus_contract(&StylusContract::Verifier).unwrap();

        // deploy precompile test contract
        let precompiles_contract_address = deploy_stylus_contract(
            &StylusContract::PrecompileTestContract,
            &value.rpc_url,
            &value.priv_key,
            client.clone(),
        )
        .await?;

        // deploy verifier contract
        let sumcheck_verifier_address = deploy_stylus_contract(
            &StylusContract::SumcheckVerifier,
            &value.rpc_url,
            &value.priv_key,
            client.clone(),
        )
        .await?;

        let shplemini_verifier_address = deploy_stylus_contract(
            &StylusContract::ShpleminiVerifier,
            &value.rpc_url,
            &value.priv_key,
            client.clone(),
        )
        .await?;

        let verifier_contract_address = deploy_stylus_contract(
            &StylusContract::Verifier,
            &value.rpc_url,
            &value.priv_key,
            client.clone(),
        )
        .await?;

        // initialize verifier contract
        let verifier_contract = VerifierContractInstance::new(verifier_contract_address, client.provider());
        let tx = verifier_contract.initialize(sumcheck_verifier_address, shplemini_verifier_address).send().await.unwrap();

        Ok(Self {
            client,
            precompiles_contract_address,
            verifier_contract_address,
        })
    }

    /// Build an instance of the darkpool contract
    pub fn verifier_contract(&self) -> VerifierTestInstance {
        VerifierTestInstance::new(self.verifier_contract_address, self.client.provider())
    }

    /// Build an instance of the precompile test contract
    pub fn precompile_test_contract(&self) -> PrecompileTestInstance {
        PrecompileTestInstance::new(self.precompiles_contract_address, self.client.provider())
    }
}

/// Defines a wrapper type around the test args so that the macro caller can
/// take inventory on the IntegrationTest type which is owned by the
/// `integration-helpers` package.
///
/// This is necessary because the inventory::collect macro implements a foreign
/// trait on the type it accepts
struct TestWrapper(IntegrationTest<TestContext>);

// Collect the statically defined tests into an iterable
inventory::collect!(TestWrapper);

/// Defines a secondary CLI that allows the test harness to simply no-op if
/// `--skip integration` is passed. This is useful when running only unit tests
/// from the workspace level, i.e.:     cargo test --workspace -- --skip
/// integration will properly skip tests run by this harness

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about=None)]
struct SkipCLI {
    /// The skip filter placed on the tests
    #[arg(long, value_parser)]
    skip: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let print_harness = match args.verbosity {
        TestVerbosity::Quiet => false,
        _ => true,
    };

    let tests_ctx = TestContext::try_new(args.clone()).await.unwrap();

    // ---------
    // | Setup |
    // ---------

    if print_harness {
        println!("\n\n{}\n", "Running integration tests...".blue());
    }

    // Call the setup callback if requested
    if matches!(args.verbosity, TestVerbosity::Verbose) {
        tracing_subscriber::fmt().pretty().init();
    }

    // ----------------
    // | Test Harness |
    // ----------------

    let mut all_success = true;
    for test_wrapper in inventory::iter::<TestWrapper>.into_iter() {
        let test = &test_wrapper.0;
        if args.test.is_some() && !test.name.contains(args.test.as_deref().unwrap()) {
            continue;
        }

        if print_harness {
            // Flush the write buffer before the test executes. We print success or
            // failure on the same line as the "Running", but if the
            // test panics, we want to know which test was run
            print!("Running {}... ", test.name);
            stdout().flush().unwrap();
        }
        let res: eyre::Result<()> = match test.test_fn {
            IntegrationTestFn::SynchronousFn(f) => f(tests_ctx.clone()),
            IntegrationTestFn::AsynchronousFn(f) => f(tests_ctx.clone()).await,
        };

        all_success &= validate_success(res, print_harness);
    }

    if all_success {
        if print_harness {
            println!("\n{}", "Integration tests successful!".green(),);
        }

        exit(0);
    }

    exit(-1);
}

/// Prints a success or failure message, returns true if success, false if
/// failure
#[inline]
fn validate_success(res: eyre::Result<()>, print_harness: bool) -> bool {
    if res.is_ok() {
        if print_harness {
            println!("{}", "Success!".green());
        }

        true
    } else {
        println!("{}\n\t{}", "Failure...".red(), res.err().unwrap());
        false
    }
}
