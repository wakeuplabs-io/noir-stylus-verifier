pub(crate) struct VerifierGenerator {}

impl VerifierGenerator {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn generate_verifier_contract(
        &self,
        circuit_json_path: String,
        vk_path: String,
    ) -> String {
        let vk_bytes_str = vk_bytes
            .iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<String>>()
            .join(", ");
        let mut inputs_prototype_str = String::new();
        let mut inputs_serialization_str = String::new();
        if !circuit_inputs.is_empty() {
            inputs_prototype_str = circuit_inputs
                .iter()
                .filter(|input| input.visibility == "public")
                .map(|input| format!("{}: Bytes", input.name))
                .collect::<Vec<String>>()
                .join(", ");
            inputs_serialization_str = circuit_inputs
                .iter()
                .filter(|input| input.visibility == "public")
                .map(|input| format!("{}.to_vec()", input.name))
                .collect::<Vec<String>>()
                .join(", ");
        };

        let circuit_json_comment = format!("/*\n * {}\n */\n", circuit_json.replace("\n", "\n * "));

        format!("

{circuit_json_comment}

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::{{abi::Bytes, prelude::*}};

#[allow(deprecated)]
use stylus_sdk::call::Call as InterfaceCall;

sol_storage! {{
    #[entrypoint]
    pub struct VerifierContract {{
        address verifier_address;
    }}
}}

sol_interface! {{
    interface IGlobalVerifier {{
        function verify(bytes memory proof, bytes memory public_inputs, bytes memory vk) external returns (bool);
    }}
}}

#[public]
impl VerifierContract {{
    #[constructor]
    pub fn constructor(&mut self, verifier_address: Address) {{
        self.verifier_address.set(verifier_address);
    }}

    pub fn verify(&mut self, proof_bytes: Bytes{inputs_prototype_str}) -> bool {{
        IGlobalVerifier::new(self.verifier_address.get()).verify(
            #[allow(deprecated)]
            InterfaceCall::new(),
            proof_bytes.to_vec().into(),
            [{inputs_serialization_str}].concat().into(),
            [{vk_bytes_str}].into(),
        ).unwrap_or(false)
    }}

    pub fn get_verifier_address(&self) -> Address {{
        self.verifier_address.get()
    }}
}}
")
    }
}

const GITIGNORE: &str = r#"
.env
target/
"#;

const RUST_TOOLCHAIN: &str = r#"
[toolchain]
channel = "1.83.0"
"#;

const CARGO_TOML: &str = r#"
[package]
name = "verifier"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
homepage = "https://github.com/wakeuplabs-io/noir-stylus-verifier"
repository = "https://github.com/wakeuplabs-io/noir-stylus-verifier"
keywords = ["arbitrum", "ethereum", "stylus", "alloy"]
description = "Stylus verifier"

[dependencies]
alloy-primitives = "=0.8.20"
alloy-sol-types = "=0.8.20"
stylus-sdk = "0.9.0"
hex = { version = "0.4", default-features = false }

[dev-dependencies]
alloy-primitives = { version = "=0.8.20", features = ["sha3-keccak"] }
tokio = { version = "1.12.0", features = ["full"] }
ethers = "2.0"
eyre = "0.6.8"
stylus-sdk = { version = "0.9.0", features = ["stylus-test"] }
dotenv = "0.15.0"

[features]
default = ["mini-alloc"]
export-abi = ["stylus-sdk/export-abi"]
debug = ["stylus-sdk/debug"]
mini-alloc = ["stylus-sdk/mini-alloc"]

[[bin]]
name = "verifier"
path = "src/main.rs"

[lib]
crate-type = ["lib", "cdylib"]

[profile.release]
codegen-units = 1
strip = true
lto = true
panic = "abort"

# If you need to reduce the binary size, it is advisable to try other
# optimization levels, such as "s" and "z"
opt-level = 3

"#;

const MAIN_RS: &str = r#"
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[cfg(not(any(test, feature = "export-abi")))]
#[no_mangle]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    verifier::print_from_args();
}
"#;
