use core::ops::{Deref, DerefMut};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    ailment::LiveAilment,
    item::{Item, ItemId},
    moves::{
        owned::OwnedMove,
        set::{OwnedMoveSet, SavedMoveSet},
        Move, MoveId, PP,
    },
    pokemon::{
        stat::{BaseStat, StatType, Stats},
        Experience, Friendship, Gender, Health, Level, Nature, Pokemon, PokemonId, EvolutionType,
    },
    Dex, Identifiable, Initializable, Uninitializable,
};

// pub type HP = crate::MaximumNumber<Health>;

/// A pokemon owned by a player.
/// This can be (de)serialized and does not borrow values.
pub type SavedPokemon =
    OwnablePokemon<PokemonId, SavedMoveSet, ItemId, Option<Gender>, Option<Nature>, Option<Health>>;

/// A pokemon owned by a player.
/// This struct has borrowed values from multiple [Dex]es.
pub type OwnedPokemon<P, M, I> = OwnablePokemon<P, OwnedMoveSet<M>, I, Gender, Nature, Health>;

/// The base struct for a pokemon owned by a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnablePokemon<P, M, I, G, N, H> {
    /// Pokemon Identifier
    pub pokemon: P,

    /// [Level] of the pokemon (1 - 100)
    pub level: Level,

    /// The [Gender] of this pokemon.
    #[serde(default)]
    pub gender: G,

    pub nature: N,

    #[serde(default)]
    /// The [Health] of this pokemon.
    pub hp: H,

    #[serde(default = "Stats::default_iv")]
    pub ivs: Stats,
    #[serde(default)]
    pub evs: Stats,

    #[serde(default = "Pokemon::default_friendship")]
    pub friendship: Friendship,

    #[serde(default)]
    pub ailment: Option<LiveAilment>,

    /// Optional nickname for the pokemon
    #[serde(default)]
    pub nickname: Option<String>,

    #[serde(default)]
    pub moves: M,

    #[serde(default = "Option::default")]
    pub item: Option<I>,

    #[serde(default)]
    pub experience: Experience,
}

impl<P, M, I, G, N, H> OwnablePokemon<P, M, I, G, N, H> {
    /// Get the current HP of this pokemon.
    pub fn hp(&self) -> H
    where
        H: Copy,
    {
        self.hp
    }
}

impl<P, M, I, G, N> OwnablePokemon<P, M, I, G, N, Health> {
    /// Has the pokemon fainted.
    pub fn fainted(&self) -> bool {
        self.hp == 0
    }
}

impl<P: Deref<Target = Pokemon>, M, I, G, N, H> OwnablePokemon<P, M, I, G, N, H> {
    /// Get the name of this pokemon.
    /// Returns the nickname or the pokemon's name.
    pub fn name(&self) -> &str {
        self.nickname
            .as_deref()
            .unwrap_or_else(|| self.pokemon.name())
    }

    pub fn should_evolve(&self) -> Option<&PokemonId> {
        match &self.pokemon.evolution {
            Some(e) => match &e.0 {
                EvolutionType::Level(level) => match level >= &self.level {
                    true => Some(&e.1),
                    false => None,
                },
                // To - do
                _ => None,
            },
            None => None,
        }
    }

    /// Get the [Experience] from this pokemon at its current [Level].
    pub fn exp_from(&self) -> Experience {
        self.pokemon.exp_from(self.level)
    }

    /// Get this pokemon's moves at its current [Level].
    pub fn moves_at_level(&self) -> impl DoubleEndedIterator<Item = &MoveId> + '_ {
        self.pokemon.moves_at_level(self.level)
    }
}

impl<P: Deref<Target = Pokemon>, M, I, G> OwnablePokemon<P, M, I, G, Nature, Health> {
    /// Get the maximum [Health] of this pokemon.
    pub fn max_hp(&self) -> Health {
        self.stat(StatType::Health)
    }

    /// Get the current [Health] of this pokemon as a percentage.
    pub fn percent_hp(&self) -> f32 {
        self.hp() as f32 / self.max_hp() as f32
    }

    /// Get a [BaseStat] for this pokemon.
    pub fn stat(&self, stat: StatType) -> BaseStat {
        self.pokemon
            .stat(&self.ivs, &self.evs, self.level, self.nature, stat)
    }

    /// Heal this pokemon with an optional amount of [Health].
    pub fn heal_hp(&mut self, amount: Option<Health>) {
        let max = self.max_hp();
        self.hp = amount.unwrap_or(max).min(max);
    }
}

impl<
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I,
        G,
        MSET: Deref<Target = [OwnedMove<M>]> + DerefMut,
    > OwnablePokemon<P, MSET, I, G, Nature, Health>
{
    /// Heal this pokemon with an optional amount of [Health] and restore all its move's [PP] by an optional amount.
    pub fn heal(&mut self, hp: Option<Health>, pp: Option<PP>) {
        self.heal_hp(hp);
        self.moves.iter_mut().for_each(|o| o.restore(pp));
    }
}

impl<P: Deref<Target = Pokemon>, M: Deref<Target = Move>, I, G, N>
    OwnablePokemon<P, OwnedMoveSet<M>, I, G, N, Health>
{
    /// Add [Experience] to this pokemon, and also handle level ups.
    pub fn add_exp<'d>(
        &mut self,
        movedex: &'d dyn Dex<'d, Move, M>,
        experience: Experience,
    ) -> impl DoubleEndedIterator<Item = &MoveId> + '_ {
        // add exp to pokemon

        self.experience += experience * 5;

        // level the pokemon up if they reach a certain amount of exp (and then subtract the exp by the maximum for the previous level)

        let previous = self.level;

        let gr = &self.pokemon.training.growth;

        while self.experience > gr.max_exp(self.level) {
            self.experience -= gr.max_exp(self.level);
            self.level += 1;
        }

        self.on_level_up(movedex, previous)
    }

    /// Handle leveling up.
    pub fn on_level_up<'d>(
        &mut self,
        movedex: &'d dyn Dex<'d, Move, M>,
        previous: Level,
    ) -> impl DoubleEndedIterator<Item = &MoveId> + '_ {
        // Get the moves the pokemon learns at the level it just gained.

        let mut moves = self.pokemon.moves_at(previous..self.level);

        // Add moves if the player's pokemon does not have a full set of moves.

        while !self.moves.is_full() {
            match moves.next() {
                Some(id) => {
                    if let Some(m) = movedex.try_get(id) {
                        self.moves.add(None, m);
                    }
                }
                None => break,
            }
        }

        moves
    }
}

impl SavedPokemon {
    pub fn fainted(&self) -> bool {
        self.hp == Some(0)
    }

    pub fn heal(&mut self, hp: Option<Health>, pp: Option<PP>) {
        self.heal_hp(hp);
        self.heal_pp(pp);
    }

    pub fn heal_hp(&mut self, hp: Option<Health>) {
        match hp {
            Some(by) => {
                if let Some(hp) = self.hp.as_mut() {
                    *hp = hp.saturating_add(by);
                }
            }
            None => self.hp = None,
        }
    }

    pub fn heal_pp(&mut self, pp: Option<PP>) {
        self.moves.iter_mut().for_each(|m| m.restore(pp))
    }

    /// Generate an owned pokemon.
    pub fn generate(
        pokemon: PokemonId,
        level: Level,
        gender: Option<Gender>,
        ivs: Option<Stats>,
    ) -> Self {
        Self {
            // data: OwnablePokemonData {
            pokemon,
            level,
            gender,
            nature: Default::default(),
            hp: Default::default(),
            ivs: ivs.unwrap_or_default(),
            evs: Default::default(),
            friendship: Pokemon::default_friendship(),
            ailment: Default::default(),
            // },
            nickname: Default::default(),
            moves: Default::default(),
            item: Default::default(),
            experience: Default::default(),
        }
    }

    /// Initialize a [SavedPokemon] that already has values given to its uninitialized fields
    pub fn try_init<
        'd,
        R: Rng,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    >(
        self,
        pokedex: &'d dyn Dex<'d, Pokemon, P>,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) -> Option<OwnedPokemon<P, M, I>> {
        let pokemon = pokedex.try_get(&self.pokemon)?;
        let gender = self.gender?;
        let nature = self.nature?;
        let hp = self.hp?;
        let moves = self.moves.init(movedex)?;
        let item = self.item.map(|ref id| itemdex.try_get(id)).flatten();
        Some(OwnablePokemon {
            // data: OwnablePokemonData {
            pokemon,
            level: self.level,
            gender,
            nature,
            hp,
            ivs: self.ivs,
            evs: self.evs,
            friendship: self.friendship,
            ailment: self.ailment,
            // },
            nickname: self.nickname,
            moves,
            item,
            experience: self.experience,
        })
    }

    /// Initialize this owned pokemon struct into an [OwnedPokemon] so it can perform more functions.
    pub fn init<
        'd,
        R: Rng,
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    >(
        self,
        random: &mut R,
        pokedex: &'d dyn Dex<'d, Pokemon, P>,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) -> Option<OwnedPokemon<P, M, I>> {
        let pokemon = pokedex.try_get(&self.pokemon)?;
        let gender = self
            .gender
            .unwrap_or_else(|| pokemon.generate_gender(random));
        let nature = self
            .nature
            .unwrap_or_else(|| Pokemon::generate_nature(random));
        let hp = self.hp.unwrap_or_else(|| {
            pokemon.stat(&self.ivs, &self.evs, self.level, nature, StatType::Health)
        });
        let mut moves = self.moves.init(movedex)?;
        if moves.is_empty() {
            for m in pokemon
                .moves_at(1..=self.level)
                .rev()
                .take(4)
                .flat_map(|id| movedex.try_get(id))
            {
                moves.add(None, m);
            }
        }
        let item = self.item.map(|ref id| itemdex.try_get(id)).flatten();
        Some(OwnablePokemon {
            // data: OwnablePokemonData {
            pokemon,
            level: self.level,
            gender,
            nature,
            hp,
            ivs: self.ivs,
            evs: self.evs,
            friendship: self.friendship,
            ailment: self.ailment,
            // },
            nickname: self.nickname,
            moves,
            item,
            experience: self.experience,
        })
    }
}

impl<
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
        G: Into<Option<Gender>>,
        N: Into<Option<Nature>>,
        H: Into<Option<Health>>,
    > Uninitializable for OwnablePokemon<P, OwnedMoveSet<M>, I, G, N, H>
{
    type Output = SavedPokemon;

    fn uninit(self) -> Self::Output {
        Self::Output {
            // data: OwnablePokemonData {
            pokemon: *self.pokemon.id(),
            level: self.level,
            gender: self.gender.into(),
            nature: self.nature.into(),
            hp: self.hp.into(),
            ivs: self.ivs,
            evs: self.evs,
            friendship: self.friendship,
            ailment: self.ailment,
            // },
            nickname: self.nickname,
            moves: self.moves.uninit(),
            item: self.item.map(|item| item.id),
            experience: self.experience,
        }
    }
}

// impl<P, M, I, H> Deref for OwnablePokemon<P, M, I, H> {
//     type Target = OwnablePokemonData<P, H>;

//     fn deref(&self) -> &Self::Target {
//         &self
//     }
// }

// impl<P, M, I, H> DerefMut for OwnablePokemon<P, M, I, H> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self
//     }
// }
