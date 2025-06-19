use serde::{Deserialize, Serialize};

pub const PRECOMPUTED_ENTITIES_SIZE: usize = 27;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PrecomputedEntities<T: Default> {
    pub elements: [T; PRECOMPUTED_ENTITIES_SIZE],
}

impl<T: Default> PrecomputedEntities<T> {
    /// column 0
    pub(crate) const Q_M: usize = 0;
    /// column 1
    pub(crate) const Q_C: usize = 1;
    /// column 2
    pub(crate) const Q_L: usize = 2;
    /// column 3
    pub(crate) const Q_R: usize = 3;
    /// column 4
    pub(crate) const Q_O: usize = 4;
    /// column 5
    pub(crate) const Q_4: usize = 5;
    /// column 6
    pub(crate) const Q_LOOKUP: usize = 6;
    /// column 7
    pub(crate) const Q_ARITH: usize = 7;
    /// column 8
    pub(crate) const Q_DELTA_RANGE: usize = 8;
    /// column 9
    pub(crate) const Q_ELLIPTIC: usize = 9;
    /// column 10
    pub(crate) const Q_AUX: usize = 10;
    /// column 11
    pub(crate) const Q_POSEIDON2_EXTERNAL: usize = 11;
    /// column 12
    pub(crate) const Q_POSEIDON2_INTERNAL: usize = 12;
    /// column 13
    const SIGMA_1: usize = 13;
    /// column 14
    const SIGMA_2: usize = 14;
    /// column 15
    const SIGMA_3: usize = 15;
    /// column 16
    const SIGMA_4: usize = 16;
    /// column 17
    const ID_1: usize = 17;
    /// column 18
    const ID_2: usize = 18;
    /// column 19
    const ID_3: usize = 19;
    /// column 20
    const ID_4: usize = 20;
    /// column 21
    const TABLE_1: usize = 21;
    /// column 22
    const TABLE_2: usize = 22;
    /// column 23
    const TABLE_3: usize = 23;
    /// column 24
    const TABLE_4: usize = 24;
    /// column 25
    const LAGRANGE_FIRST: usize = 25;
    /// column 26
    const LAGRANGE_LAST: usize = 26;

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements.iter_mut()
    }

    pub fn q_m(&self) -> &T {
        &self.elements[Self::Q_M]
    }

    pub fn q_c(&self) -> &T {
        &self.elements[Self::Q_C]
    }

    pub fn q_l(&self) -> &T {
        &self.elements[Self::Q_L]
    }

    pub fn q_r(&self) -> &T {
        &self.elements[Self::Q_R]
    }

    pub fn q_o(&self) -> &T {
        &self.elements[Self::Q_O]
    }

    pub fn q_4(&self) -> &T {
        &self.elements[Self::Q_4]
    }

    pub fn q_arith(&self) -> &T {
        &self.elements[Self::Q_ARITH]
    }

    pub fn q_delta_range(&self) -> &T {
        &self.elements[Self::Q_DELTA_RANGE]
    }

    pub fn q_elliptic(&self) -> &T {
        &self.elements[Self::Q_ELLIPTIC]
    }

    pub fn q_aux(&self) -> &T {
        &self.elements[Self::Q_AUX]
    }

    pub fn q_lookup(&self) -> &T {
        &self.elements[Self::Q_LOOKUP]
    }

    pub fn q_poseidon2_external(&self) -> &T {
        &self.elements[Self::Q_POSEIDON2_EXTERNAL]
    }

    pub fn q_poseidon2_internal(&self) -> &T {
        &self.elements[Self::Q_POSEIDON2_INTERNAL]
    }

    pub fn sigma_1(&self) -> &T {
        &self.elements[Self::SIGMA_1]
    }

    pub fn sigma_2(&self) -> &T {
        &self.elements[Self::SIGMA_2]
    }

    pub fn sigma_3(&self) -> &T {
        &self.elements[Self::SIGMA_3]
    }

    pub fn sigma_4(&self) -> &T {
        &self.elements[Self::SIGMA_4]
    }

    pub fn id_1(&self) -> &T {
        &self.elements[Self::ID_1]
    }

    pub fn id_2(&self) -> &T {
        &self.elements[Self::ID_2]
    }

    pub fn id_3(&self) -> &T {
        &self.elements[Self::ID_3]
    }

    pub fn id_4(&self) -> &T {
        &self.elements[Self::ID_4]
    }

    pub fn table_1(&self) -> &T {
        &self.elements[Self::TABLE_1]
    }

    pub fn table_2(&self) -> &T {
        &self.elements[Self::TABLE_2]
    }

    pub fn table_3(&self) -> &T {
        &self.elements[Self::TABLE_3]
    }

    pub fn table_4(&self) -> &T {
        &self.elements[Self::TABLE_4]
    }

    pub fn lagrange_first(&self) -> &T {
        &self.elements[Self::LAGRANGE_FIRST]
    }

    pub fn lagrange_last(&self) -> &T {
        &self.elements[Self::LAGRANGE_LAST]
    }

}
