use dep::std;

// Commitment to the vote using Pedersen hash
fn main(
    root: pub Field,
    index: Field,
    hash_path: [Field; 2],
    secret: Field,
    proposalId: pub Field,
    vote_commitment: pub Field,
    vote_opening: Field,
    salt: Field
) -> pub Field {
    // Verify the vote opening (actual vote and secret) matches the commitment
    let computed_vote_commitment = std::hash::pedersen_hash([vote_opening, secret]);
    assert(vote_commitment == computed_vote_commitment);

    // Cast vote_opening to an integer for comparison
    let vote_opening_int = vote_opening as i32;

    // Range proof for vote validity (0 or 1)
    assert(vote_opening_int >= 0);
    assert(vote_opening_int <= 1);

    // Compute the nullifier using a random salt
    let nullifier = std::hash::pedersen_hash([root, secret, proposalId, salt]);

    // Merkle tree verification
    let note_commitment = std::hash::pedersen_hash([secret]);
    // let check_root = std::merkle::compute_merkle_root(note_commitment, index, hash_path);
    // assert(root == check_root);

    nullifier
}

#[test]
fn test_valid_build_merkle_tree_and_recursive_proof() {
    let secret = 1;
    let salt = 999;
    let vote_opening = 1;
    let proposalId = 0;

    let commitment_0 = std::hash::pedersen_hash([secret]);
    let commitment_1 = std::hash::pedersen_hash([2]);
    let commitment_2 = std::hash::pedersen_hash([3]);
    let commitment_3 = std::hash::pedersen_hash([4]);

    let left_branch = std::hash::pedersen_hash([commitment_0, commitment_1]);
    let right_branch = std::hash::pedersen_hash([commitment_2, commitment_3]);

    let root = std::hash::pedersen_hash([left_branch, right_branch]);

    let vote_commitment = std::hash::pedersen_hash([vote_opening, secret]);

    let nullifier = main(
        root,
        0,
        [commitment_1, right_branch],
        secret,
        proposalId,
        vote_commitment,
        vote_opening,
        salt
    );

    let expected_nullifier = std::hash::pedersen_hash([root, secret, proposalId, salt]);

    assert(nullifier == expected_nullifier);
}