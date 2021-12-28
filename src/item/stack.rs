use core::ops::Deref;
use serde::{Deserialize, Serialize};

use crate::{
    item::{Item, ItemId, Stackable},
    Dex, Initializable, Uninitializable,
};

pub type SavedItemStack = ItemStack<ItemId>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemStack<I> {
    pub item: I,
    pub count: usize,
}

impl<I> From<I> for ItemStack<I> {
    fn from(item: I) -> Self {
        Self { item, count: 0 }
    }
}

impl<I: Clone> ItemStack<I> {
    fn take_gt(&mut self, count: usize) -> Self {
        self.count -= count;
        Self {
            item: self.item.clone(),
            count,
        }
    }

    pub fn try_take(&mut self, count: usize) -> Option<Self> {
        if count > self.count {
            None
        } else {
            Some(self.take_gt(count))
        }
    }

    pub fn take(&mut self, count: usize) -> Self {
        if count > self.count {
            let stack = Self {
                item: self.item.clone(),
                count: self.count,
            };
            self.count = 0;
            stack
        } else {
            self.take_gt(count)
        }
    }
}

impl<I: Deref<Target = Item>> ItemStack<I> {
    pub fn try_use(&mut self) -> bool {
        if self.count > 0 {
            if self.item.should_consume() {
                self.count -= 1;
            }
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, count: usize) -> bool {
        if matches!(self.item.stackable, Stackable::Unique) && self.count != 0 {
            return false;
        }
        self.count = self.count.saturating_add(count);
        true
    }

    pub fn stacks(&self) -> usize {
        match self.item.stackable {
            Stackable::Unique | Stackable::Singular => self.count,
            Stackable::Stackable(size) => self.count / size as usize,
        }
    }
}

impl<'d, O: Deref<Target = Item>> Initializable<'d, Item, O> for SavedItemStack {
    type Output = ItemStack<O>;

    fn init(self, dex: &'d dyn Dex<'d, Item, O>) -> Option<Self::Output> {
        Some(Self::Output {
            item: dex.try_get(&self.item)?,
            count: self.count,
        })
    }
}

impl<I: Deref<Target = Item>> Uninitializable for ItemStack<I> {
    type Output = SavedItemStack;

    fn uninit(self) -> Self::Output {
        Self::Output {
            item: self.item.id,
            count: self.count,
        }
    }
}
