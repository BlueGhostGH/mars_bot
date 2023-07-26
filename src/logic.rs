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

    pub fn magic(&self) -> GameOutput {
        let ores = self
            .map
            .tiles
            .iter()
            .enumerate()
            .flat_map(|(x, array)| {
                array
                    .iter()
                    .enumerate()
                    .filter_map(|(y, tile)| match tile {
                        Tile::Osmium | Tile::Iron => Some((x, y)),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
    }
}
