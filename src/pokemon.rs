//! Types and structs related to Pokemon
//!

use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::RangeBounds,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    moves::{MoveCategory, MoveId},
    types::{Effective, PokemonType},
    Identifiable,
};

pub mod species;

pub mod owned;

pub mod party;

pub mod data;
use self::data::*;

pub mod stat;
use self::stat::{BaseStat, Stat, StatType, Stats};

/// The identifier of a Pokemon.
pub type PokemonId = u16;
/// The level of a pokemon. Usually 1 - 100.
/// Levels determine a Pokemon's power, and higher is better.
pub type Level = u8;
/// How much experience a Pokemon has.
/// Experience is progress between a Pokemon's levels.
pub type Experience = u32;
/// The friendship value of a Pokemon. 0 - 255.
pub type Friendship = u8;
/// The amount of health a pokemon has.
pub type Health = stat::BaseStat;

/// A Pokemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pokemon {
    pub id: <Self as Identifiable>::Id,
    pub name: String,

    pub primary_type: PokemonType,
    #[serde(default)]
    pub secondary_type: Option<PokemonType>,

    #[serde(default)]
    pub moves: Vec<LearnableMove>,
    pub base: Stats,

    pub species: String,

    #[serde(default)]
    pub evolution: Option<Evolution>,

    pub height: u8,
    pub weight: u16,

    pub training: Training,
    pub breeding: Breeding,
}

impl Pokemon {
    /// Generate a pokemon's [Gender] based on its percent to be a certain gender and a random number generator.
    pub fn generate_gender(&self, random: &mut impl Rng) -> Gender {
        self.breeding
            .gender
            .map(
                |percentage| match random.gen_range(Gender::RANGE) > percentage {
                    true => Gender::Male,
                    false => Gender::Female,
                },
            )
            .unwrap_or(Gender::None)
    }

    /// Test how [Effective] a [PokemonType] is on this pokemon, in a specified [MoveCategory].
    pub fn effective(&self, user: PokemonType, category: MoveCategory) -> Effective {
        let primary = user.effective(self.primary_type, category);
        if let Some(secondary) = self.secondary_type {
            primary * user.effective(secondary, category)
        } else {
            primary
        }
    }

    /// Get the amount of [Experience] that can be gained from defeating this pokemon at a certain [Level].
    pub const fn exp_from(&self, level: Level) -> Experience {
        ((self.training.base_exp * level as u16) / 7) as Experience
    }

    /// Get the moves of a pokemon at a certain [Level].
    pub fn moves_at_level(&self, level: Level) -> impl DoubleEndedIterator<Item = &MoveId> + '_ {
        self.moves_at(level..=level)
    }

    /// Get an iterator of the moves a pokemon can get from a range of levels.
    pub fn moves_at<'s, R: RangeBounds<Level> + 's>(
        &'s self,
        levels: R,
    ) -> impl DoubleEndedIterator<Item = &'s MoveId> + 's {
        self.moves
            .iter()
            .filter(move |m| levels.contains(&m.0))
            .map(|m| &m.1)
    }

    /// Get the value of a [BaseStat] from basic stats.
    pub fn stat(&self, ivs: &Stats, evs: &Stats, level: Level, stat: StatType) -> BaseStat {
        match stat {
            StatType::Health => Self::base_hp(self.base.hp, ivs.hp, evs.hp, level),
            stat => Self::base_stat(self.base.get(stat), ivs.get(stat), evs.get(stat), level),
        }
    }

    /// Get the value of a [BaseStat] from basic stats, excluding health.
    pub fn base_stat(base: Stat, iv: Stat, ev: Stat, level: Level) -> BaseStat {
        //add item check
        let nature = 1.0;
        (((2.0 * base as f32 + iv as f32 + ev as f32) * level as f32 / 100.0 + 5.0).floor()
            * nature)
            .floor() as BaseStat
    }

    /// Get the base [Health] of a pokemon from basic stats.
    pub fn base_hp(base: Stat, iv: Stat, ev: Stat, level: Level) -> Health {
        ((2.0 * base as f32 + iv as f32 + ev as f32) * level as f32 / 100.0 + level as f32 + 10.0)
            .floor() as Health
    }

    /// The default [Friendship] of a pokemon.
    pub const fn default_friendship() -> Friendship {
        70
    }
}

impl Identifiable for Pokemon {
    type Id = PokemonId;

    const UNKNOWN: Self::Id = 0;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Pokemon {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "#{} {}", self.id, self.name)
    }
}

#[test]
fn tests() {
    use crate::{
        moves::{Move, MoveCategory, MoveTarget, Power, PP},
        pokemon::{owned::SavedPokemon, stat::StatSet},
        BasicDex,
    };

    let mut pokedex = BasicDex::default();

    let test = "test".parse().unwrap();

    let v = Pokemon {
        id: 0,
        name: "Test".to_owned(),
        primary_type: PokemonType::Bug,
        secondary_type: Some(PokemonType::Dragon),
        moves: vec![LearnableMove(1, test)],
        base: StatSet::uniform(60),
        species: "Test Species".to_owned(),
        evolution: None,
        height: 6_5,
        weight: 100,
        training: Training {
            base_exp: 200,
            growth: Default::default(),
        },
        breeding: Breeding { gender: None },
    };

    pokedex.insert(v);

    let mut movedex = BasicDex::default();

    let v = Move {
        id: test,
        name: "Test Move".to_owned(),
        category: MoveCategory::Physical,
        pokemon_type: PokemonType::Bug,
        accuracy: None,
        power: Some(Power::MAX),
        pp: PP::MAX,
        priority: 0,
        target: MoveTarget::Opponent,
        contact: false,
        crit_rate: 1,
    };

    movedex.insert(v);

    let itemdex = BasicDex::default();

    let mut rng = rand::rngs::mock::StepRng::new(12, 24);

    let pokemon = SavedPokemon::generate(&mut rng, 0, 30, None, None);

    let pokemon = pokemon
        .init(&mut rng, &pokedex, &movedex, &itemdex)
        .unwrap();

    assert!(pokemon.moves.len() != 0)
}
