use core::ops::{Deref, DerefMut};

use crate::{
    moves::{
        owned::{OwnedMove, SavedMove},
        Move,
    },
    Dex, Initializable, Uninitializable,
};

pub const MOVE_SET_SIZE: usize = 4;

type Set<T> = arrayvec::ArrayVec<[T; MOVE_SET_SIZE]>;

pub type SavedMoveSet = Set<SavedMove>;

impl<'d, O: Deref<Target = Move>> Initializable<'d, Move, O> for SavedMoveSet {
    type Output = OwnedMoveSet<O>;

    fn init(self, dex: &'d dyn Dex<'d, Move, O>) -> Option<Self::Output> {
        Some(OwnedMoveSet(
            self.into_iter().flat_map(|s| s.init(dex)).collect(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct OwnedMoveSet<M: Deref<Target = Move>>(Set<OwnedMove<M>>);

impl<M: Deref<Target = Move>> OwnedMoveSet<M> {
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    pub fn add(&mut self, index: Option<usize>, m: M) {
        let m = OwnedMove::from(m);
        match self.0.is_full() {
            true => {
                if let Some(i) = index.map(|i| self.0.get_mut(i)).flatten() {
                    *i = m
                }
            }
            false => self.0.push(m),
        }
    }
}

impl<M: Deref<Target = Move>> Uninitializable for OwnedMoveSet<M> {
    type Output = SavedMoveSet;

    fn uninit(self) -> Self::Output {
        self.0.into_iter().map(|o| o.uninit()).collect()
    }
}

impl<M: Deref<Target = Move>> Default for OwnedMoveSet<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: Deref<Target = Move>> Deref for OwnedMoveSet<M> {
    type Target = [OwnedMove<M>];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<M: Deref<Target = Move>> DerefMut for OwnedMoveSet<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
