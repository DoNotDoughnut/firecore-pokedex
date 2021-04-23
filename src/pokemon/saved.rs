use serde::{Deserialize, Serialize};

use super::status::PokemonStatus;
use super::{Level, PokemonId, Experience, Friendship, data::Gender, data::StatSet};

use crate::item::ItemId;
use crate::moves::saved::SavedMoveSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPokemon {

    pub id: PokemonId,

    pub data: PokemonData,

    #[serde(default)]
    pub item: Option<ItemId>,
    #[serde(default)]
    pub moves: Option<SavedMoveSet>,
    #[serde(default)]
    pub current_hp: Option<u16>,
    #[serde(default)]
    pub owned_data: Option<OwnedPokemon>,
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonData {

    #[serde(default)]
    pub nickname: Option<String>,
    pub level: Level,
    pub gender: Gender,

    // #[serde(default)]
    // pub ability: Option<Ability>,
    
    #[serde(default = "default_iv")]
	pub ivs: StatSet,
    #[serde(default)]
    pub evs: StatSet,

    #[serde(default)]
	pub experience: Experience,

    #[serde(default = "default_friendship")]
    pub friendship: Friendship,

    #[serde(default)]
    pub status: Option<PokemonStatus>,

    // #[serde(default)]
    // pub item: Option<Item>, // item: struct with name, texture, description, and singular script-like enum which activates function of item

    // #[serde(default)]

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedPokemon {

    pub original_trainer: String,
    pub original_location: (String, Level),

}

pub const fn default_iv() -> StatSet {
    StatSet::uniform(15)
}

pub const fn default_friendship() -> Friendship {
    70
}