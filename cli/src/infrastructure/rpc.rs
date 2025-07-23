use reqwest::Client;
use serde_json::json;
use std::{error::Error, future::Future, pin::Pin};

#[derive(Default)]
pub(crate) struct Rpc {}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TRpc: Send + Sync + 'static {
    fn get_chain_id(
        &self,
        rpc_url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, Box<dyn Error>>> + Send>>;
}

// implementations ==========================================

impl TRpc for Rpc {
    fn get_chain_id(
        &self,
        rpc_url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, Box<dyn Error>>> + Send>> {
        let rpc_url = rpc_url.to_string();

        Box::pin(async move {
            let client = Client::new();
            let res = client
                .post(&rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "eth_chainId",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let hex = res["result"]
                .as_str()
                .ok_or("Missing result from eth_chainId")?;

            let chain_id = u64::from_str_radix(hex.trim_start_matches("0x"), 16)?;
            Ok(chain_id)
        })
    }
}
