use alloc::vec::Vec;

#[derive(Clone, Debug, Default)]
pub struct Polynomial<F> {
    pub coefficients: Vec<F>,
}