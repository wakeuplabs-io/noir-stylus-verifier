//! Definitions of errors that can occur during the execution of the contract
//! management scripts

/// Errors that can occur during the execution of the contract management
/// scripts
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// Error initializing the RPC client
    #[error("error initializing client: {0}")]
    ClientInitialization(String),
    /// Error fetching the nonce of the deployer
    #[error("error fetching nonce: {0}")]
    NonceFetching(String),
    /// Error deploying a contract
    #[error("error deploying contract: {0}")]
    ContractDeployment(String),
    /// Error calling a contract method
    #[error("error interacting with contract: {0}")]
    ContractInteraction(String),
    /// Error compiling a Stylus contract
    #[error("error compiling contract: {0}")]
    ContractCompilation(String),
}
