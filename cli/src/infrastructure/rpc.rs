//! # RPC Client
//!
//! This module provides RPC client functionality for interacting with blockchain
//! networks. It uses the Alloy library to communicate with Ethereum-compatible
//! JSON-RPC endpoints for operations like chain ID detection.

use alloy::providers::{Provider, ProviderBuilder};
use async_trait::async_trait;
use std::error::Error;

/// RPC client for blockchain interactions.
/// 
/// This struct provides functionality to interact with Ethereum-compatible
/// blockchain networks via JSON-RPC, primarily for chain detection and
/// validation during deployment operations.
#[derive(Default)]
pub(crate) struct Rpc {}

/// Trait defining the interface for RPC operations.
/// 
/// This trait abstracts blockchain RPC interactions to enable testing
/// and different RPC provider implementations.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(crate) trait TRpc {
    /// Retrieves the chain ID from a blockchain RPC endpoint.
    /// 
    /// Makes an RPC call to determine the chain ID of the target blockchain,
    /// which is used to select appropriate default verifier contract addresses.
    /// 
    /// # Arguments
    /// 
    /// * `rpc_url` - The RPC endpoint URL to query
    /// 
    /// # Returns
    /// 
    /// Returns the chain ID as a u64, or an error if the RPC call fails
    /// or the URL is invalid.
    async fn get_chain_id(&self, rpc_url: &str) -> Result<u64, Box<dyn Error>>;
}

// implementations ==========================================

#[async_trait]
impl TRpc for Rpc {
    /// Implements chain ID retrieval using the Alloy provider.
    /// 
    /// Creates an HTTP provider for the given RPC URL and calls the
    /// `eth_chainId` RPC method to determine the blockchain's chain ID.
    async fn get_chain_id(&self, rpc_url: &str) -> Result<u64, Box<dyn Error>> {
        let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

        let chain_id = provider.get_chain_id().await?;
        Ok(chain_id)
    }
}
