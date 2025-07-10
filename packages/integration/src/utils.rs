//! Utilities for the deploy scripts.

use crate::{errors::ScriptError, types::StylusContract};
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{DynProvider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use colored::Colorize;
use eyre::Result;
use std::{
    borrow::Borrow,
    env,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};
use regex::Regex;

/// An Ethers provider that uses a `LocalWallet` to generate signatures
/// & interfaces with the RPC endpoint over HTTP
#[derive(Clone)]
pub struct LocalWalletHttpClient {
    /// The RPC url
    url: Url,
    /// The underlying provider
    provider: DynProvider<Ethereum>,
    /// The signer
    signer: PrivateKeySigner,
}

impl Borrow<DynProvider<Ethereum>> for LocalWalletHttpClient {
    fn borrow(&self) -> &DynProvider<Ethereum> {
        &self.provider
    }
}

impl LocalWalletHttpClient {
    /// Creates a new LocalWalletHttpClient
    pub fn new(signer: PrivateKeySigner, url: Url) -> Self {
        let eth_wallet = EthereumWallet::from(signer.clone());
        let provider = ProviderBuilder::new()
            .wallet(eth_wallet)
            .on_http(url.clone());
        Self {
            url,
            provider: DynProvider::new(provider),
            signer,
        }
    }

    /// Return a copy of the RPC url
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    /// Return a reference to the underlying provider
    pub fn provider(&self) -> DynProvider<Ethereum> {
        self.provider.clone()
    }

    /// Returns the signer
    pub fn signer(&self) -> &PrivateKeySigner {
        &self.signer
    }

    /// Returns the address of the signer
    pub fn address(&self) -> Address {
        self.signer.address()
    }
}

/// Sets up the address and client with which to instantiate a contract for
/// testing, reading in the private key, RPC url, and contract address from the
/// environment.
pub async fn setup_client(
    priv_key: &str,
    rpc_url: &str,
) -> Result<LocalWalletHttpClient, ScriptError> {
    let url = Url::parse(rpc_url).map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;
    let signer = PrivateKeySigner::from_str(priv_key)
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    Ok(LocalWalletHttpClient::new(signer, url))
}

/// Executes a command, returning an error if the command fails
fn command_success_or(mut cmd: Command, err_msg: &str) -> Result<String, ScriptError> {
    let output = cmd
        .output()
        .map_err(|e| ScriptError::ContractCompilation(e.to_string()))?;
    
    if !output.status.success() {
        Err(ScriptError::ContractCompilation(String::from(err_msg)))
    } else {
        println!("Output: {}", String::from_utf8(output.stdout.clone()).unwrap());
        String::from_utf8(output.stdout)
            .map_err(|e| ScriptError::ContractCompilation(e.to_string()))
    }
}

/// Compiles the given Stylus contract to WASM and optimizes the resulting
/// binary, returning the path to the optimized WASM file.
///
/// Assumes that `cargo`, the `nightly` toolchain, and `wasm-opt` are locally
/// available.
#[allow(dead_code)]
pub fn build_stylus_contract(contract: &StylusContract) -> Result<PathBuf, ScriptError> {
    println!("{}", format!("Building contract {contract}...").blue());

    let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_path = current_dir
        .ancestors()
        .nth(2)
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find root directory",
        )))?;

    // Build the contracts
    let mut build_cmd = Command::new("just");
    build_cmd.stdout(Stdio::null()).stderr(Stdio::null());
    build_cmd.arg("build-contract");
    build_cmd.arg(contract.to_string());
    build_cmd.current_dir(workspace_path);

    command_success_or(build_cmd, "Failed to build contracts")?;

    // Optimize the WASM file
    let mut optimize_cmd = Command::new("just");
    optimize_cmd.stdout(Stdio::null()).stderr(Stdio::null());
    optimize_cmd.arg("optimize-contract");
    optimize_cmd.arg(contract.to_string());
    optimize_cmd.current_dir(workspace_path);

    command_success_or(optimize_cmd, "Failed to optimize contracts")?;

    let wasm_file_path = workspace_path.join(format!(
        "target/wasm32-unknown-unknown/release/{contract}-opt.wasm"
    ));

    Ok(wasm_file_path)
}


/// Deploys the given compiled Stylus contract, saving its deployment address
pub async fn deploy_stylus_contract(
    contract: &StylusContract,
    rpc_url: &str,
    priv_key: &str,
    constructor_signature: &str,
    constructor_args: &[String],
) -> Result<Address, ScriptError> {
    println!("{}", format!("Deploying contract {contract}...").blue());

    let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_path = current_dir
        .ancestors()
        .nth(2)
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find root directory",
        )))?;

    let mut deploy_cmd = Command::new("just");
    deploy_cmd.arg("--set");
    deploy_cmd.arg("rpc_url");
    deploy_cmd.arg(rpc_url);
    deploy_cmd.arg("--set");
    deploy_cmd.arg("private_key");
    deploy_cmd.arg(priv_key);
    deploy_cmd.arg("deploy-contract");
    deploy_cmd.arg(contract.to_string());

    if !constructor_signature.is_empty() {
        deploy_cmd.arg(constructor_signature);
        deploy_cmd.args(constructor_args);
    }

    deploy_cmd.current_dir(workspace_path);

    let out = command_success_or(deploy_cmd, "Failed to deploy contract")
        .map_err(|e| ScriptError::ContractDeployment(e.to_string()))?;

    let out_stripped = strip_color(&out);
    let deployed_address = extract_deployed_address(&out_stripped)?;

    Address::from_str(deployed_address).map_err(|e| ScriptError::ContractDeployment(e.to_string()))
}


fn strip_color(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*[ABCDHJKSTfGmsu]").unwrap();
    re.replace_all(s, "").into_owned()
}

fn extract_deployed_address(s: &str) -> Result<&str, ScriptError> {
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("deployed code at address: ") {
            return Ok(rest.split(" ").next().unwrap());
        }
    }

    Err(ScriptError::ContractDeployment("deployment address not found".to_string()))
}