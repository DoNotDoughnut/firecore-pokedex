use core::ops::Deref;

use crate::Identifiable;

/// A Dex is used to hold types with an identifiable value (see [Identifiable]).
pub trait Dex<I: Identifiable> {
    type Output: Deref<Target = I>;

    /// Try to get an identifiable value from the Dex.
    fn try_get(&self, id: &I::Id) -> Option<&Self::Output>;

    /// Get the unknown value from the Dex.
    fn unknown(&self) -> &Self::Output;

    /// Get the identifiable value from the Dex, or return the unknown value.
    fn get(&self, id: &I::Id) -> &Self::Output {
        self.try_get(id).unwrap_or_else(|| self.unknown())
    }

    /// Get the length of the Dex.
    fn len(&self) -> usize;

    /// Check if the Dex is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub use defaults::BasicDex;

mod defaults {

    use core::{hash::Hash, ops::Deref};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use hashbrown::HashMap;

    use crate::Identifiable;

    use super::Dex;

    fn name<T: ?Sized>() -> &'static str {
        let name = core::any::type_name::<T>();
        name.split("::").last().unwrap_or(name)
    }

    /// Basic Dex implementation using a HashMap.
    #[repr(transparent)]
    #[derive(Debug, Clone)]
    pub struct BasicDex<I: Identifiable, O: Deref<Target = I> + From<I>>(pub HashMap<I::Id, O>)
    where
        I::Id: Hash + Eq;

    impl<I: Identifiable, O: Deref<Target = I> + From<I>> BasicDex<I, O>
    where
        I::Id: Hash + Eq,
    {
        pub fn new(inner: HashMap<I::Id, O>) -> Self {
            Self(inner)
        }

        pub fn insert(&mut self, v: I) -> Option<O>
        where
            I::Id: Clone,
        {
            self.0.insert(v.id().clone(), O::from(v))
        }

        pub fn into_inner(self) -> HashMap<I::Id, O> {
            self.0
        }

        pub fn try_get_named(&self, name: &str) -> Option<&O> {
            self.0
                .values()
                .find(|i| i.name().eq_ignore_ascii_case(name))
        }

        pub fn get_mut(&mut self, id: &I::Id) -> Option<&mut O> {
            self.0.get_mut(id)
        }
    }

    impl<I: Identifiable, O: Deref<Target = I> + Clone + From<I>> Dex<I> for BasicDex<I, O>
    where
        I::Id: Hash + Eq,
    {
        type Output = O;

        fn try_get(&self, id: &I::Id) -> Option<&O> {
            self.0.get(id)
        }

        fn unknown(&self) -> &O {
            self.try_get(&I::UNKNOWN).unwrap_or_else(|| {
                panic!(
                    "Could not get unknown {} for \"{}\"",
                    name::<I>(),
                    name::<Self>()
                )
            })
        }

        fn len(&self) -> usize {
            self.0.len()
        }
    }

    impl<I: Identifiable, O: Deref<Target = I> + Clone + From<I>> Default for BasicDex<I, O>
    where
        I::Id: Hash + Eq,
    {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    /// Serialize Dex as a Vec
    impl<I: Identifiable + Serialize, O: Deref<Target = I> + Clone + From<I>> Serialize
        for BasicDex<I, O>
    where
        I::Id: Hash + Eq,
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_seq(self.0.values().map(Deref::deref))
        }
    }

    /// Deserialize Dex from a Vec
    impl<'de, I: Identifiable + Deserialize<'de>, O: Deref<Target = I> + Clone + From<I>>
        Deserialize<'de> for BasicDex<I, O>
    where
        I::Id: Hash + Eq + Clone,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            alloc::vec::Vec::<I>::deserialize(deserializer).map(|i| {
                Self(
                    i.into_iter()
                        .map(|i| (i.id().clone(), O::from(i)))
                        .collect(),
                )
            })
        }
    }
}
