use dashmap::DashMap as HashMap;

use firecore_pokedex_lib::{
	pokemon::{PokemonId, Pokemon},
	moves::{MoveId, PokemonMove}
};

pub use firecore_pokedex_lib::serialized;

pub mod pokemon;
pub mod moves;

pub type Pokedex = HashMap<PokemonId, Pokemon>;
pub type Movedex = HashMap<MoveId, PokemonMove>;

pub static mut POKEDEX: Option<Pokedex> = None;
pub static mut MOVEDEX: Option<Movedex> = None;

pub fn pokedex() -> &'static Pokedex {
	unsafe { POKEDEX.as_ref().unwrap() }
}

pub fn movedex() -> &'static Movedex {
	unsafe { MOVEDEX.as_ref().unwrap() }
}

pub fn new() {
	unsafe {
		POKEDEX = Some(HashMap::new());
		MOVEDEX = Some(HashMap::new());
	}
}