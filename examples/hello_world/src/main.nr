fn main(x: Field, y: pub Field, z: pub Field) {
    assert(x != y);
    assert(x != z);
}

#[test]
fn test_main() {
    main(1, 2, 3);

    // Uncomment to make test fail
    // main(1, 1, 3);
}
