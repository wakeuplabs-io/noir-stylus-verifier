use alloy_sol_types::sol;

sol! {

    // -------------
    // | FUNCTIONS |
    // -------------

    function verify(bytes proof, bytes public_inputs, bytes vk) external returns (bool);
}