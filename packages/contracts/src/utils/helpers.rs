use alloc::vec::Vec;
use alloy_sol_types::{SolCall, SolType};
use stylus_sdk::{alloy_primitives::Address, call::MutatingCallContext};
use crate::utils::constants::CALL_RETDATA_DECODING_ERROR_MESSAGE;

#[allow(deprecated)]
use stylus_sdk::call::call;


/// Performs a `call` to the given address, calling the function
/// defined as a `SolCall` with the given arguments.
#[allow(deprecated)]
pub fn call_helper<C: SolCall>(
    storage: impl MutatingCallContext,
    address: Address,
    args: <C::Parameters<'_> as SolType>::RustType,
) -> Result<C::Return, Vec<u8>> {
    let calldata = C::new(args).abi_encode();
    let res = call(storage, address, &calldata).map_err(map_call_error)?;
    C::abi_decode_returns(&res, false /* validate */)
        .map_err(|_| CALL_RETDATA_DECODING_ERROR_MESSAGE.to_vec())
}

/// Maps an error returned from an external contract call to a `Vec<u8>`,
/// which is the expected return type of external contract methods.
#[allow(deprecated)]
pub fn map_call_error(e: stylus_sdk::call::Error) -> Vec<u8> {
    match e {
        stylus_sdk::call::Error::Revert(msg) => msg,
        stylus_sdk::call::Error::AbiDecodingFailed(_) => {
            CALL_RETDATA_DECODING_ERROR_MESSAGE.to_vec()
        }
    }
}
