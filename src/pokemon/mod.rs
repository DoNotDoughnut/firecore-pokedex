use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::Range,
};

use crate::{
    id::{Dex, Identifiable, IdentifiableRef},
    moves::{MoveCategory, MoveId, UninitMoveSet},
    pokemon::{
        data::{Breeding, Gender, LearnableMove, PokedexData, Training},
        stat::{BaseStatSet, BaseStat, StatType, Stats},
    },
    types::{Effective, PokemonType},
};

use arrayvec::ArrayVec;
use rand::Rng;
use serde::{Deserialize, Serialize};

mod instance;

pub use instance::*;

pub mod data;
pub mod stat;

pub type PokemonId = <Pokemon as Identifiable>::Id;
pub type Level = u8;
pub type Experience = u32;
pub type Friendship = u8;
pub type Health = stat::BaseStat;

#[derive(Serialize, Deserialize)]
pub struct Pokemon {
    pub id: PokemonId,
    pub name: String,

    pub primary_type: PokemonType,
    pub secondary_type: Option<PokemonType>,

    pub base: Stats,

    pub data: PokedexData,

    pub training: Training,
    pub breeding: Breeding,

    pub moves: Vec<LearnableMove>,
}

pub type Party<P> = ArrayVec<[P; 6]>;

impl Pokemon {
    pub fn generate_moves(&self, level: Level) -> UninitMoveSet {
        let mut learnable = self
            .moves
            .iter()
            .filter(|learnable_move| learnable_move.level <= level)
            .map(|learnable_move| learnable_move.id)
            .rev();
        
        let mut moves = UninitMoveSet::new();

        while !moves.is_full() {
            match learnable.next() {
                Some(m) => if !moves.iter().any(|i| i.m == m) {
                    moves.push(m.into());
                }
                None => break,
            }
        }

        moves

    }

    pub fn generate_gender(&self, random: &mut impl Rng) -> Option<Gender> {
        self.breeding.gender.map(|percentage| match random.gen_range(Gender::RANGE) > percentage {
            true => Gender::Male,
            false => Gender::Female,
        })
    }

    pub fn effective(&self, user: PokemonType, category: MoveCategory) -> Effective {
        let primary = user.effective(self.primary_type, category);
        if let Some(secondary) = self.secondary_type {
            primary * user.effective(secondary, category)
        } else {
            primary
        }
    }

    pub fn exp_from(&self, level: Level) -> Experience {
        ((self.training.base_exp * level as u16) / 7) as Experience
    }

    pub fn moves_at_level(&self, level: Level) -> impl Iterator<Item = MoveId> + '_ {
        self.moves
            .iter()
            .filter(move |m| m.level == level)
            .map(|l| l.id)
    }

    pub fn moves_at(&self, levels: Range<Level>) -> impl Iterator<Item = MoveId> + '_ {
        let levels = Range {
            start: levels.start + 1,
            end: levels.end + 1,
        };

        levels
            .into_iter()
            .flat_map(move |level| self.moves_at_level(level))
    }

    pub fn stat(&self, ivs: &Stats, evs: &Stats, level: Level, stat: StatType) -> BaseStat {
        match stat {
            StatType::Health => {
                BaseStatSet::hp(self.base.hp, ivs.hp, evs.hp, level)
            }
            StatType::Attack => {
                BaseStatSet::stat(self.base.atk, ivs.atk, evs.atk, level)
            }
            StatType::Defense => {
                BaseStatSet::stat(self.base.def, ivs.def, evs.def, level)
            }
            StatType::SpAttack => BaseStatSet::stat(
                self.base.sp_atk,
                ivs.sp_atk,
                evs.sp_atk,
                level,
            ),
            StatType::SpDefense => BaseStatSet::stat(
                self.base.sp_def,
                ivs.sp_def,
                evs.sp_def,
                level,
            ),
            StatType::Speed => BaseStatSet::stat(
                self.base.speed,
                ivs.speed,
                evs.speed,
                level,
            ),
        }
    }

    pub const fn default_friendship() -> Friendship {
        70
    }
}

impl Identifiable for Pokemon {
    type Id = u16;

    const UNKNOWN: Self::Id = 0;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

pub type PokemonRef<'a> = IdentifiableRef<'a, Pokemon>;

pub type Pokedex = Dex<Pokemon>;

impl Debug for Pokemon {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self, f)
    }
}

impl Display for Pokemon {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} ({})", self.name, self.id)
    }
}
