#[macro_export]
macro_rules! must_deser {
    ($type:ty, $bytes:expr) => {
        <$type as ultrahonk::serialize::BytesDeserializable>::deserialize_from_bytes(
            $bytes.as_ref(),
        )
        .unwrap()
        .0
    };
}
