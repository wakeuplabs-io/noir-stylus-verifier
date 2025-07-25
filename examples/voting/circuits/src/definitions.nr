use poseidon::poseidon2::Poseidon2::hash;

pub fn commitment(private_key: Field, secret: Field) -> Field {
    hash([private_key, secret], 2)
}

pub fn nullifier(root: Field, priv_key: Field, proposal_id: Field) -> Field {
    hash([root, priv_key, proposal_id], 3)
}

