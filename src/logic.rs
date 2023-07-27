use crate::game::{
    input::{GameInput, Map, PlayerInventory, PlayerStats, ShittyPosition, Tile},
    output::{Action, Direction, GameOutput, Moves, Upgrade},
};

#[derive(Debug)]
struct CachedPlayerStats {
    pub gun_level: u8,
    pub wheel_level: u8,
}

#[derive(Debug)]
pub struct CachedPlayer {
    up_to_date: bool,
    position: ShittyPosition,
    stats: CachedPlayerStats,
}

#[derive(Debug)]
pub struct GameState {
    pub map: Map,
    pub cached_player: Option<CachedPlayer>,
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
        dbg!(input.player_stats.wheel_level);
        let enemy = input
            .map
            .tiles
            .iter()
            .map(|column| column.iter())
            .flatten()
            .enumerate()
            .find_map(|(i, entry)| match entry.tile {
                Tile::Player { .. } => Some(CachedPlayer {
                    up_to_date: true,
                    position: ShittyPosition {
                        x: (i / input.map.dimensions.height as usize) as i8,
                        y: (i % input.map.dimensions.height as usize) as i8,
                    },
                    stats: CachedPlayerStats {
                        gun_level: 1,
                        wheel_level: 1,
                    },
                }),
                _ => None,
            });
        let mut result = Self {
            base_position: input.map.player_position,
            map: input.map,
            cached_player: enemy,
            player_stats: input.player_stats,
            player_inventory: input.player_inventory,
            upgrade_queue_index: 0,
        };

        result
            .map
            .set_tile_at(result.map.player_position, Tile::Air);

        result
            .map
            .floodfill(result.player_stats.wheel_level as usize);

        result
    }

    fn feed_input(&mut self, input: GameInput) {
        dbg!(input.player_stats.wheel_level);

        let enemy = input
            .map
            .tiles
            .iter()
            .map(|column| column.iter())
            .flatten()
            .enumerate()
            .find_map(|(i, entry)| match entry.tile {
                Tile::Player { .. } => Some(CachedPlayer {
                    up_to_date: true,
                    position: ShittyPosition {
                        x: (i / input.map.dimensions.height as usize) as i8,
                        y: (i % input.map.dimensions.height as usize) as i8,
                    },
                    stats: CachedPlayerStats {
                        gun_level: 1,
                        wheel_level: 1,
                    },
                }),
                _ => None,
            });
        self.cached_player.as_mut().map(|cached_enemy| match enemy {
            Some(new_enemy) => *cached_enemy = new_enemy,
            None => cached_enemy.up_to_date = false,
        });

        self.map
            .merge_with(&input.map, input.player_stats.wheel_level as usize);
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

    fn move_towards(&self, to: ShittyPosition) -> (Moves, ShittyPosition, Option<Direction>) {
        self.map
            .move_towards(to, self.player_stats.wheel_level as usize)
    }

    fn moves(&self) -> (Moves, ShittyPosition, Option<Direction>) {
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
        let (moves, new_position, optional_mining_direction) = self.moves();
        let neighbour = self
            .map
            .find_neighbour(new_position, Tile::Osmium)
            .or_else(|| self.map.find_neighbour(new_position, Tile::Iron))
            .or_else(|| self.map.find_neighbour(new_position, Tile::Stone));

        let action = if let Some(direction) = optional_mining_direction {
            Some(Action::Mine { direction })
        } else if let Some((direction, _)) = neighbour {
            Some(Action::Mine { direction })
        } else {
            None
        };

        let upgrade = match self.target_upgrade() {
            Some(target_upgrade)
                if self
                    .player_inventory
                    .can_afford(target_upgrade.cost(self.player_stats))
                    && self.can_upgrade(new_position) =>
            {
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
