use alloy::providers::{Provider, ProviderBuilder};
use async_trait::async_trait;
use std::error::Error;

#[derive(Default)]
pub(crate) struct Rpc {}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(crate) trait TRpc {
    async fn get_chain_id(&self, rpc_url: &str) -> Result<u64, Box<dyn Error>>;
}

// implementations ==========================================

#[async_trait]
impl TRpc for Rpc {
    async fn get_chain_id(&self, rpc_url: &str) -> Result<u64, Box<dyn Error>> {
        let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

        let chain_id = provider.get_chain_id().await?;
        Ok(chain_id)
    }
}
