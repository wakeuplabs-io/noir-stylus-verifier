mod definitions;
mod merkle_tree;

use poseidon::poseidon2::Poseidon2::hash;

global DEPTH: u32 = 2;

fn main(
    root: pub Field,
    path: [Field; DEPTH],
    direction_selector: [bool; DEPTH],
    secret: Field,
    priv_key: Field,
    nullifier: pub Field,
    proposal_id: pub Field,
    vote: pub u8,
) {
    let note_commitment = definitions::commitment(priv_key, secret);

    merkle_tree::assert_merkle_inclusion(
        root,
        path,
        direction_selector,
        note_commitment,
        "Commitment not in Merkle Tree",
    );
    assert(nullifier == definitions::nullifier(root, priv_key, proposal_id), "Invalid nullifier");
    assert(vote <= 1, "Invalid vote");
}

#[test]
fn test_valid_vote_yes() {
    // Test data setup
    let priv_key = 123456789;
    let secret = 987654321;
    let proposal_id = 1;
    let vote = 1; // YES vote
    
    // Create commitment
    let note_commitment = definitions::commitment(priv_key, secret);
    
    // Build a simple merkle tree with depth 2
    // Tree structure:
    //       root
    //      /    \
    //   left    right
    //   /  \    /   \
    // com   0  0     0
    let sibling = definitions::commitment(0, 0); // dummy commitment
    let left_branch = hash([note_commitment, sibling], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    // Path and direction for leftmost leaf
    let path = [sibling, right_branch];
    let direction_selector = [false, false]; // left, left
    
    let nullifier = definitions::nullifier(root, priv_key, proposal_id);
    
    // This should pass
    main(root, path, direction_selector, secret, priv_key, nullifier, proposal_id, vote);
}

#[test]
fn test_valid_vote_no() {
    // Test data setup
    let priv_key = 555666777;
    let secret = 111222333;
    let proposal_id = 2;
    let vote = 0; // NO vote
    
    let note_commitment = definitions::commitment(priv_key, secret);
    
    // Build merkle tree - place commitment at index 1 (right leaf of left branch)
    let sibling = definitions::commitment(0, 0);
    let left_branch = hash([sibling, note_commitment], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    let path = [sibling, right_branch];
    let direction_selector = [true, false]; // right, left
    
    let nullifier = definitions::nullifier(root, priv_key, proposal_id);
    
    main(root, path, direction_selector, secret, priv_key, nullifier, proposal_id, vote);
}

#[test(should_fail_with = "Invalid vote")]
fn test_invalid_vote_value() {
    let priv_key = 123456789;
    let secret = 987654321;
    let proposal_id = 1;
    let vote = 2; // Invalid vote (must be 0 or 1)
    
    let note_commitment = definitions::commitment(priv_key, secret);
    let sibling = definitions::commitment(0, 0);
    let left_branch = hash([note_commitment, sibling], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    let path = [sibling, right_branch];
    let direction_selector = [false, false];
    let nullifier = definitions::nullifier(root, priv_key, proposal_id);
    
    main(root, path, direction_selector, secret, priv_key, nullifier, proposal_id, vote);
}

#[test(should_fail_with = "Invalid nullifier")]
fn test_invalid_nullifier() {
    let priv_key = 123456789;
    let secret = 987654321;
    let proposal_id = 1;
    let vote = 1;
    
    let note_commitment = definitions::commitment(priv_key, secret);
    let sibling = definitions::commitment(0, 0);
    let left_branch = hash([note_commitment, sibling], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    let path = [sibling, right_branch];
    let direction_selector = [false, false];
    
    // Wrong nullifier - using different proposal_id
    let wrong_nullifier = definitions::nullifier(root, priv_key, proposal_id + 1);
    
    main(root, path, direction_selector, secret, priv_key, wrong_nullifier, proposal_id, vote);
}

#[test(should_fail_with = "Commitment not in Merkle Tree")]
fn test_invalid_merkle_proof() {
    let priv_key = 123456789;
    let secret = 987654321;
    let proposal_id = 1;
    let vote = 1;
    
    let note_commitment = definitions::commitment(priv_key, secret);
    let sibling = definitions::commitment(0, 0);
    
    // Create a tree where the commitment is NOT included
    let other_commitment = definitions::commitment(999, 888);
    let left_branch = hash([other_commitment, sibling], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    // But provide path as if our commitment was included
    let path = [sibling, right_branch];
    let direction_selector = [false, false];
    let nullifier = definitions::nullifier(root, priv_key, proposal_id);
    
    main(root, path, direction_selector, secret, priv_key, nullifier, proposal_id, vote);
}

#[test(should_fail_with = "Invalid nullifier")]
fn test_nullifier_with_wrong_private_key() {
    let priv_key = 123456789;
    let secret = 987654321;
    let proposal_id = 1;
    let vote = 1;
    
    let note_commitment = definitions::commitment(priv_key, secret);
    let sibling = definitions::commitment(0, 0);
    let left_branch = hash([note_commitment, sibling], 2);
    let right_branch = hash([sibling, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    let path = [sibling, right_branch];
    let direction_selector = [false, false];
    
    // Generate nullifier with wrong private key
    let wrong_nullifier = definitions::nullifier(root, priv_key + 1, proposal_id);
    
    main(root, path, direction_selector, secret, priv_key, wrong_nullifier, proposal_id, vote);
}

#[test]
fn test_different_tree_positions() {
    // Test commitment at different positions in the tree
    let priv_key = 777888999;
    let secret = 444555666;
    let proposal_id = 3;
    let vote = 0;
    
    let note_commitment = definitions::commitment(priv_key, secret);
    let sibling = definitions::commitment(0, 0);
    
    // Place commitment at index 2 (left leaf of right branch)
    let left_branch = hash([sibling, sibling], 2);
    let right_branch = hash([note_commitment, sibling], 2);
    let root = hash([left_branch, right_branch], 2);
    
    let path = [sibling, left_branch];
    let direction_selector = [false, true]; // left, right
    let nullifier = definitions::nullifier(root, priv_key, proposal_id);
    
    main(root, path, direction_selector, secret, priv_key, nullifier, proposal_id, vote);
}
