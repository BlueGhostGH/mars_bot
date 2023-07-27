use std::{dbg, println};

use crate::game::{
    input::{GameInput, Map, PlayerInventory, PlayerStats, ShittyPosition, Tile},
    output::{Action, GameOutput, Moves, Upgrade},
};

#[derive(Debug)]
pub struct GameState {
    pub map: Map,
    pub player_stats: PlayerStats,
    pub player_inventory: PlayerInventory,
    pub upgrade_queue_index: usize,
    pub base_position: ShittyPosition,
}

impl GameState {
    pub fn process_input(previous: Option<GameState>, input: GameInput) -> Self {
        match previous {
            None => Self::from_input(input),
            Some(mut previous) => {
                previous.feed_input(input);
                previous
            }
        }
    }

    fn from_input(input: GameInput) -> Self {
        let mut result = Self {
            base_position: input.map.player_position,
            map: input.map,
            player_stats: input.player_stats,
            player_inventory: input.player_inventory,
            upgrade_queue_index: 0,
        };

        result
            .map
            .set_tile_at(result.map.player_position, Tile::Air);

        result.map.floodfill();

        result
    }

    fn feed_input(&mut self, input: GameInput) {
        self.map.merge_with(&input.map);
        self.player_stats = input.player_stats;
        self.player_inventory = input.player_inventory;
    }

    fn target_upgrade(&self) -> Option<Upgrade> {
        if self.player_stats.hit_points <= 3 {
            Some(Upgrade::Heal)
        } else {
            Upgrade::UPGRADE_QUEUE
                .get(self.upgrade_queue_index)
                .copied()
        }
    }

    fn can_upgrade(&self, position: ShittyPosition) -> bool {
        position == self.base_position || self.player_stats.has_battery
    }

    fn move_towards(&self, to: ShittyPosition) -> (Moves, ShittyPosition) {
        self.map.move_towards(to, self.player_stats.wheel_level)
    }

    fn moves(&self) -> (Moves, ShittyPosition) {
        match self.target_upgrade() {
            Some(target_upgrade)
                if self
                    .player_inventory
                    .can_afford(target_upgrade.cost(self.player_stats))
                    && !self.can_upgrade(self.map.player_position) =>
            {
                return self.move_towards(self.base_position);
            }
            _ => {}
        };

        let closest = self
            .map
            .closest_tile(Tile::Osmium)
            .or_else(|| self.map.closest_tile(Tile::Iron));

        return if let Some(closest) = closest {
            println!(
                "going for a known: {closest:?} {:?}",
                self.map.tile_at(closest)
            );
            self.move_towards(closest)
        } else {
            println!("going for unknown");
            let unknown = self.map.closest_tile(Tile::Unknown).unwrap();
            dbg!(self.map.player_position);
            dbg!(unknown);
            self.move_towards(unknown)
        };
    }

    pub fn magic(&mut self) -> GameOutput {
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

        dbg!(self.can_upgrade(new_position));
        dbg!(self
            .player_inventory
            .can_afford(self.target_upgrade().unwrap().cost(self.player_stats)));

        let upgrade = match self.target_upgrade() {
            Some(target_upgrade)
                if self
                    .player_inventory
                    .can_afford(target_upgrade.cost(self.player_stats))
                    && self.can_upgrade(new_position) =>
            {
                dbg!(target_upgrade);
                if target_upgrade != Upgrade::Heal {
                    self.upgrade_queue_index += 1;
                }

                Some(target_upgrade)
            }
            _ => None,
        };

        GameOutput {
            moves: Some(moves),
            action,
            upgrade,
        }
    }
}
