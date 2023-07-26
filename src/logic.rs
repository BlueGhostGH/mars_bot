use std::todo;

use crate::game::{
    input::{Dimensions, GameInput, Map, Tile},
    output::{Direction, GameOutput},
};

#[derive(Debug)]
pub struct GameState {
    pub dimensions: Dimensions,
    pub map: Map,
}

impl GameState {
    pub fn feed_input(&mut self, input: GameInput) {
        self.dimensions = input.map.dimensions;
        self.map.merge_with(&input.map);
    }

    pub fn magic(&self) -> GameOutput {
        let closest = self
            .map
            .closest_tile(Tile::Osmium)
            .or_else(|| self.map.closest_tile(Tile::Iron));

        if let Some(closest) = closest {
            self.map.move_towards(closest)
        } else {
            let unknown = self.map.closest_tile(Tile::Unknown).unwrap();
            self.map.move_towards(unknown)
        }
    }
}
