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
        let ores: Vec<(usize, usize)> = self
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
                    .collect::<Vec<(usize, usize)>>()
            })
            .collect();

        let closest = ores.iter().min_by_key(|position| self.map.distance_to(**position));
        if let Some(closest) = closest {
            map.move_towards(closest)
        } else {

        }
    }
}
