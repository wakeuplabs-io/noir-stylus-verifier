use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::call::Call;
use stylus_sdk::{abi::Bytes, prelude::*};
use stylus_sdk::{
    call::RawCall,
    prelude::{sol_interface, sol_storage},
};
use ultrahonk::serialize::{BytesDeserializable, BytesSerializable};
use ultrahonk::{
    decider::{
        sumcheck::round_verifier::SumcheckVerifierRound,
        types::{ClaimedEvaluations, RelationParameters},
    },
    types::ScalarField,
};
use alloy_primitives::address;

sol_interface! {
    interface ISumcheckVerifierRound {
        function compute_full_relation_purported_value(bytes memory extended_edges, bytes memory relation_parameters, bytes memory scaling_factor) external returns (bytes memory);
    }
}

#[cfg_attr(feature = "sumcheck-verifier-round", entrypoint)]
#[storage]
pub struct SumcheckVerifierRoundContract {

}

pub struct SumcheckVerifierRoundContractWrapper;

impl SumcheckVerifierRound for SumcheckVerifierRoundContractWrapper {
    fn compute_full_relation_purported_value(
        extended_edges: &ClaimedEvaluations,
        relation_parameters: &RelationParameters,
        scaling_factor: &ScalarField,
    ) -> ScalarField {
        let res_xy_bytes = ISumcheckVerifierRound::new(address!("0x0000000000000000000000000000000000000000"))
            .compute_full_relation_purported_value(
                Call::new(),
                extended_edges.serialize_to_bytes().into(),
                relation_parameters.serialize_to_bytes().into(),
                scaling_factor.serialize_to_bytes().into(),
            )
            .unwrap();

        ScalarField::deserialize_from_bytes(&res_xy_bytes).unwrap()
    }
}
