//! Definitions of errors that can occur during the execution of the contract
//! management scripts

/// Errors that can occur during the execution of the contract management
/// scripts
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// Generic error
    #[error("error: {0}")]
    Generic(String),
    /// Error deserializing a transcript
    #[error("reading command output: {0}")]
    CommandOutput(String),
    /// Error initializing the RPC client
    #[error("error initializing client: {0}")]
    ClientInitialization(String),
    /// Error deploying a contract
    #[error("error deploying contract: {0}")]
    ContractDeployment(String),
    /// Error compiling a Stylus contract
    #[error("error compiling contract: {0}")]
    ContractCompilation(String),
}
