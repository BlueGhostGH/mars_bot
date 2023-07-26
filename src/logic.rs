use std::todo;

use crate::game::{
    input::{Dimensions, GameInput, Map},
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
}

pub fn magic(game_state: &mut GameState) -> GameOutput {
    todo!();
}
