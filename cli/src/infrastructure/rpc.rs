use ethers::providers::{Http, Middleware, Provider};
use std::{error::Error, future::Future, pin::Pin};

pub(crate) struct Rpc {}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TRpc: Send + Sync + 'static {
    fn get_chain_id(
        &self,
        rpc_url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, Box<dyn Error>>> + Send>>;
}

// implementations ==========================================

impl Rpc {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl TRpc for Rpc {
    fn get_chain_id(
        &self,
        rpc_url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, Box<dyn Error>>> + Send>> {
        let rpc_url = rpc_url.to_string();

        Box::pin(async move {
            let provider = Provider::<Http>::try_from(rpc_url.as_str())?;
            let chain_id = provider.get_chainid().await?.as_u64();
            Ok(chain_id)
        })
    }
}
