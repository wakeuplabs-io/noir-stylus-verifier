/// An enum used for using the right table in the garbled circuit implementation of SHA256.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SHA256Table {
    /// Picks the CHOOSE_NORMALIZATION_TABLE (see plookup.rs)
    Choose,
    /// Picks the MAJORITY_NORMALIZATION_TABLE (see plookup.rs)
    Majority,
    /// Picks the WITNESS_EXTENSION_NORMALIZATION_TABLE (see plookup.rs)
    WitnessExtension,
}
