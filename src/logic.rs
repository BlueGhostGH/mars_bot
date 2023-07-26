use std::todo;

use crate::game::{
    input::{Dimensions, GameInput, Map, PlayerInventory, PlayerPosition, PlayerStats, Tile},
    output::{Action, Direction, GameOutput, Moves, Upgrade},
};

#[derive(Debug)]
pub struct GameState {
    pub dimensions: Dimensions,
    pub map: Map,
    pub player_stats: PlayerStats,
    pub player_inventory: PlayerInventory,
    pub player_position: PlayerPosition,
}

impl GameState {
    pub fn feed_input(&mut self, input: GameInput) {
        self.dimensions = input.map.dimensions;
        self.map.merge_with(&input.map);
        self.player_stats = input.player_stats;
        self.player_inventory = input.player_inventory;
        self.player_position = input.player_position;
    }

    pub fn target_upgrade(&self) -> Upgrade {
        todo!()
    }

    fn moves(&self) -> (Option<Moves>, (usize, usize)) {
        if self
            .player_inventory
            .can_afford(self.target_upgrade().cost())
        {
            let base = self.map.closest_tile(Tile::Base).unwrap();

            if !self.player_stats.has_battery && base != self.player_position.as_vec() {
                return self.map.move_towards(base);
            }
        }

        let closest = self
            .map
            .closest_tile(Tile::Osmium)
            .or_else(|| self.map.closest_tile(Tile::Iron));

        return if let Some(closest) = closest {
            self.map.move_towards(closest)
        } else {
            let unknown = self.map.closest_tile(Tile::Unknown).unwrap();
            self.map.move_towards(unknown)
        };
    }

    pub fn magic(&self) -> GameOutput {
        let (moves, new_position) = self.moves();
        let neighbour = self
            .map
            .find_neighbour(new_position, Tile::Osmium)
            .or_else(|| self.map.find_neighbour(new_position, Tile::Iron))
            .or_else(|| self.map.find_neighbour(new_position, Tile::Stone));

        let action = if let Some((direction, _)) = neighbour {
            Some(Action::Mine { direction })
        } else {
            None
        };

        let base = self.map.closest_tile(Tile::Base).unwrap();
        let upgrade = if self
            .player_inventory
            .can_afford(self.target_upgrade().cost())
            && (new_position == base || self.player_stats.has_battery)
        {
            Some(self.target_upgrade())
        } else {
            None
        };

        GameOutput {
            moves,
            action,
            upgrade,
        }
    }
}
