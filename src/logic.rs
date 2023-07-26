use crate::game::{
    input::{GameInput, Map, PlayerInventory, PlayerStats, ShittyPosition, Tile},
    output::{Action, GameOutput, Moves, Upgrade},
};

#[derive(Debug)]
pub struct GameState {
    pub map: Map,
    pub player_stats: PlayerStats,
    pub player_inventory: PlayerInventory,
    pub player_position: ShittyPosition,
}

impl GameState {
    pub fn from_input(input: GameInput) -> Self {
        Self {
            map: input.map,
            player_stats: input.player_stats,
            player_inventory: input.player_inventory,
            player_position: input.player_position,
        }
    }

    pub fn feed_input(&self, input: GameInput) -> Self {
        self.map.merge_with(&input.map);
        self.player_stats = input.player_stats;
        self.player_inventory = input.player_inventory;
        self.player_position = input.player_position;
    }

    pub fn target_upgrade(&self) -> Upgrade {
        let stats = self.player_stats;
        if !stats.has_battery {
            Upgrade::Battery
        } else if stats.wheel_level < 3 {
            Upgrade::Movement
        } else if stats.drill_level < 3 {
            Upgrade::Drill
        } else if stats.gun_level < 3 {
            Upgrade::Attack
        } else {
            Upgrade::Radar
        }
    }

    fn moves(&self) -> (Option<Moves>, ShittyPosition) {
        if self
            .player_inventory
            .can_afford(self.target_upgrade().cost(self.player_stats))
        {
            let base = self
                .map
                .closest_tile(self.player_position, Tile::Base)
                .unwrap();

            if !self.player_stats.has_battery && base != self.player_position {
                return self.map.move_towards(base);
            }
        }

        let closest = self
            .map
            .closest_tile(self.player_position, Tile::Osmium)
            .or_else(|| self.map.closest_tile(self.player_position, Tile::Iron));

        return if let Some(closest) = closest {
            self.map.move_towards(closest)
        } else {
            let unknown = self
                .map
                .closest_tile(self.player_position, Tile::Unknown)
                .unwrap();
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

        let base = self.map.closest_tile(new_position, Tile::Base).unwrap();
        let upgrade = if self
            .player_inventory
            .can_afford(self.target_upgrade().cost(self.player_stats))
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
