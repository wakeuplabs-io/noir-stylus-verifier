//! Definitions of errors that can occur during the execution of the contract
//! management scripts

/// Errors that can occur during the execution of the contract management
/// scripts
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// Invalid arguments provided to a command
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    /// Error reading from the deployments file
    #[error("error reading deployments: {0}")]
    ReadFile(String),
    /// Error writing to the deployments file
    #[error("error writing deployments: {0}")]
    WriteFile(String),
    /// Error parsing a Solidity compilation artifact
    #[error("error parsing artifact: {0}")]
    ArtifactParsing(String),
    /// Error initializing the RPC client
    #[error("error initializing client: {0}")]
    ClientInitialization(String),
    /// Error fetching the nonce of the deployer
    #[error("error fetching nonce: {0}")]
    NonceFetching(String),
    /// Error constructing calldata for a contract method
    #[error("error constructing calldata: {0}")]
    CalldataConstruction(String),
    /// Error deploying a contract
    #[error("error deploying contract: {0}")]
    ContractDeployment(String),
    /// Error calling a contract method
    #[error("error interacting with contract: {0}")]
    ContractInteraction(String),
    /// Error compiling a Stylus contract
    #[error("error compiling contract: {0}")]
    ContractCompilation(String),
    /// Error de/serializing calldata
    #[error("error de/serializing calldata: {0}")]
    Serde(String),
    /// Error converting between relayer and contract types
    #[error("error converting between types")]
    ConversionError,
    /// Error creating a circuit
    #[error("error creating circuit")]
    CircuitCreation,
    /// Error parsing the protocol public encryption key
    #[error("error parsing protocol pubkey: {0}")]
    PubkeyParsing(String),
}
