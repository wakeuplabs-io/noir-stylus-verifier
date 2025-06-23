//! Utilities for the deploy scripts.

use colored::Colorize;
use crate::{constants::MANIFEST_DIR_ENV_VAR, errors::ScriptError, types::StylusContract};
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use alloy_contract::{CallBuilder, CallDecoder};
use alloy_primitives::Bytes;
use eyre::Result;
use serde::Serialize;
use std::{
    borrow::Borrow,
    env,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

/// The call builder type used in the scripts
pub type EthereumCall<'a, C> = CallBuilder<(), &'a DynProvider, C, Ethereum>;

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

/// Sends a contract call, waiting for the transaction to go from pending to
/// executed, and returns the transaction receipt
pub async fn send_tx<C: CallDecoder + Unpin>(
    call: EthereumCall<'_, C>,
) -> Result<TransactionReceipt, ScriptError> {
    let pending_tx = call
        .send()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    Ok(receipt)
}

/// Sends a transaction request, waiting for the transaction to go from pending
/// to executed, and returns the transaction receipt
pub async fn send_raw_tx(
    provider: &DynProvider<Ethereum>,
    tx: TransactionRequest,
) -> Result<TransactionReceipt, ScriptError> {
    let pending_tx = provider
        .send_transaction(tx)
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    Ok(receipt)
}

/// Send a call and return the result
pub async fn call_helper<C: CallDecoder + Unpin>(
    call: EthereumCall<'_, C>,
) -> Result<C::CallOutput, ScriptError> {
    let res = call
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    Ok(res)
}

/// Executes a command, returning an error if the command fails
fn command_success_or(mut cmd: Command, err_msg: &str) -> Result<(), ScriptError> {
    if !cmd
        .output()
        .map_err(|e| ScriptError::ContractCompilation(e.to_string()))?
        .status
        .success()
    {
        Err(ScriptError::ContractCompilation(String::from(err_msg)))
    } else {
        Ok(())
    }
}

/// Compiles the given Stylus contract to WASM and optimizes the resulting
/// binary, returning the path to the optimized WASM file.
///
/// Assumes that `cargo`, the `nightly` toolchain, and `wasm-opt` are locally
/// available.
pub fn build_stylus_contract(contract: &StylusContract) -> Result<PathBuf, ScriptError> {
    println!("{}", format!("Building contract {:?}...", contract).blue());

    let current_dir = PathBuf::from(env::var(MANIFEST_DIR_ENV_VAR).unwrap());
    let workspace_path = current_dir
        .ancestors()
        .nth(2)
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find contracts directory",
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
        "target/wasm32-unknown-unknown/release/{}-opt.wasm",
        contract
    ));

    Ok(wasm_file_path)
}

/// Deploys the given compiled Stylus contract, saving its deployment address
pub async fn deploy_stylus_contract(
    contract: &StylusContract,
    rpc_url: &str,
    priv_key: &str,
    client: LocalWalletHttpClient,
) -> Result<Address, ScriptError> {
    println!("{}", format!("Deploying contract {:?}...", contract).blue());

    let current_dir = PathBuf::from(env::var(MANIFEST_DIR_ENV_VAR).unwrap());
    let workspace_path = current_dir
        .ancestors()
        .nth(2)
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find contracts directory",
        )))?;

    // Compute the expected deployment address
    let deployer_address = client.address();
    let deployer_nonce = client
        .provider()
        .get_transaction_count(deployer_address)
        .await
        .map_err(|e| ScriptError::NonceFetching(e.to_string()))?;
    let deployed_address = deployer_address.create(deployer_nonce);

    let mut deploy_cmd = Command::new("just");
    deploy_cmd.stdout(Stdio::null()).stderr(Stdio::null());
    deploy_cmd.arg("deploy-contract");
    deploy_cmd.arg(contract.to_string());
    deploy_cmd.arg(rpc_url);
    deploy_cmd.arg(priv_key);
    deploy_cmd.current_dir(workspace_path);

    command_success_or(deploy_cmd, "Failed to deploy contracts")?;

    Ok(deployed_address)
}

pub fn serialize_to_calldata<T: Serialize>(t: &T) -> Result<Bytes> {
    Ok(postcard::to_allocvec(t)?.into())
}
