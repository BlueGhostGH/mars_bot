use std::{dbg, println};

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
    pub turn: usize,
    pub final_phase_entryway: Option<(Direction, ShittyPosition)>,
    pub cage_step: usize,
}

impl GameState {
    const ACID_START_TURN: usize = 4;
    const ACID_TICK_RATE: usize = 2;

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
            turn: 0,
            final_phase_entryway: None,
            cage_step: 0,
        };

        result
            .map
            .set_tile_at(result.map.player_position, Tile::Air);

        result.map.floodfill(
            result.player_stats.wheel_level as usize,
            result.opponent_exact_position(),
        );

        result
    }

    fn feed_input(&mut self, input: GameInput) {
        let enemy = input
            .map
            .tiles
            .iter()
            .map(|column| column.iter())
            .flatten()
            .enumerate()
            .find_map(|(i, entry)| match entry.tile {
                Tile::Player { .. } => {
                    let new_position = ShittyPosition {
                        x: (i / input.map.dimensions.height as usize) as i8,
                        y: (i % input.map.dimensions.height as usize) as i8,
                    };

                    let new_gun_level = {
                        let damage_taken =
                            self.player_stats.hit_points - input.player_stats.hit_points;
                        if damage_taken > 0 {
                            let distance = if self.map.player_position.y == new_position.y {
                                self.map.player_position.x.abs_diff(new_position.x)
                            } else {
                                self.map.player_position.y.abs_diff(new_position.y)
                            };

                            damage_taken + distance - 1
                        } else {
                            self.cached_player
                                .as_ref()
                                .map(|enemy| enemy.stats.gun_level)
                                .unwrap_or(1)
                        }
                    };

                    let new_wheel_level = {
                        if let Some(CachedPlayer {
                            up_to_date: true,
                            position,
                            ..
                        }) = self.cached_player
                        {
                            let manhattan_distance = new_position.x.abs_diff(position.x)
                                + new_position.y.abs_diff(position.y);

                            manhattan_distance
                        } else {
                            self.cached_player
                                .as_ref()
                                .map(|enemy| enemy.stats.wheel_level)
                                .unwrap_or(1)
                        }
                    };

                    Some(CachedPlayer {
                        up_to_date: true,
                        position: new_position,
                        stats: CachedPlayerStats {
                            gun_level: new_gun_level,
                            wheel_level: new_wheel_level,
                        },
                    })
                }
                _ => None,
            });
        self.cached_player.as_mut().map(|cached_enemy| match enemy {
            Some(new_enemy) => *cached_enemy = new_enemy,
            None => cached_enemy.up_to_date = false,
        });

        self.map.set_acid_level(self.acid_level());
        self.map.merge_with(
            &input.map,
            input.player_stats.wheel_level as usize,
            self.opponent_exact_position(),
        );
        self.player_stats = input.player_stats;
        self.player_inventory = input.player_inventory;
        self.turn += 1;
        // dbg!(self.acid_level());
    }

    fn acid_level(&self) -> usize {
        if self.turn < Self::ACID_START_TURN {
            0
        } else {
            (self.turn - Self::ACID_START_TURN) / Self::ACID_TICK_RATE + 1
        }
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

    fn opponent_exact_position(&self) -> Option<ShittyPosition> {
        if let Some(opponent) = &self.cached_player {
            if opponent.up_to_date {
                return Some(opponent.position);
            }
        }

        return None;
    }

    fn moves(&mut self) -> (Moves, ShittyPosition, Option<Direction>) {
        if self.acid_level() > 0 {
            let center = self.map.center();

            let (direction_from_center, closest) = self
                .map
                .neighbours(center)
                .iter()
                .min_by_key(|(_, position)| self.map.distance_to(*position))
                .copied()
                .unwrap();

            if closest == self.map.player_position && self.final_phase_entryway.is_none() {
                println!("SETTING CENTER ENTRYWAY!!!");
                self.final_phase_entryway = Some((direction_from_center, closest));
            }

            if let Some((direction_from_center, closest)) = self.final_phase_entryway {
                println!("CAGE MODE {closest:?}");
                let third_spot = center.add_direction(!direction_from_center);

                if self.cage_step == 4 {
                    println!("DETECTING CAGE ALREADY BUILT");
                    if let Some(opponent_position) = self.opponent_exact_position() {
                        return self.move_towards(opponent_position);
                    }
                } else if self.map.player_position == closest
                    && self.map.tile_at(center).unwrap().tile == Tile::Air
                    && self
                        .map
                        .tile_at(closest.add_direction(direction_from_center.clockwise()))
                        .unwrap()
                        .tile
                        .is_stone()
                    && self
                        .map
                        .tile_at(closest.add_direction(direction_from_center.counterclockwise()))
                        .unwrap()
                        .tile
                        .is_stone()
                {
                    println!("BUILD MOVE 1");
                    self.cage_step = 1;
                    return self.move_towards(center);
                } else if self.map.player_position == center
                    && self.map.tile_at(third_spot).unwrap().tile == Tile::Air
                    && self
                        .map
                        .tile_at(center.add_direction(direction_from_center.clockwise()))
                        .unwrap()
                        .tile
                        == Tile::Air
                    && self
                        .map
                        .tile_at(center.add_direction(direction_from_center.counterclockwise()))
                        .unwrap()
                        .tile
                        == Tile::Air
                {
                    if self.cage_step == 1 {
                        self.cage_step = 2;
                        println!("BUILD MOVE 2");
                        return self.move_towards(third_spot);
                    } else {
                        self.cage_step = 4;
                        return (Moves::default(), self.map.player_position, None);
                    }
                } else if self.map.player_position == third_spot
                    && self
                        .map
                        .tile_at(third_spot.add_direction(direction_from_center.clockwise()))
                        .unwrap()
                        .tile
                        .is_stone()
                    && self
                        .map
                        .tile_at(third_spot.add_direction(direction_from_center.counterclockwise()))
                        .unwrap()
                        .tile
                        .is_stone()
                {
                    self.cage_step = 3;
                    println!("BUILD MOVE 3");
                    return self.move_towards(center);
                } else {
                    if self.cage_step == 3 {
                        self.cage_step = 4;
                    }

                    return (Moves::default(), self.map.player_position, None);
                }
            }

            return self.move_towards(closest);
        };

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
            // dbg!(self.map.player_position);
            // dbg!(unknown);
            self.move_towards(unknown)
        };
    }

    pub fn magic(&mut self) -> GameOutput {
        let mut run_away = None;
        let attack = if let (
            Some(CachedPlayer {
                up_to_date: true,
                position,
                stats:
                    CachedPlayerStats {
                        gun_level: enemy_gun_level,
                        ..
                    },
            }),
            true,
        ) = (&self.cached_player, self.cage_step != 4)
        {
            let GameState {
                map: Map {
                    player_position, ..
                },
                player_stats: PlayerStats { gun_level, .. },
                ..
            } = *self;

            let mut free_neighbours = self.map.find_neighbours(player_position, Tile::Air);
            free_neighbours.extend(self.map.find_neighbours(player_position, Tile::Base));
            let neighbours = self.map.neighbours(player_position);
            let enemy_around = neighbours.iter().find(|&&(_, neighbour)| {
                matches!(
                    self.map.tile_at(neighbour).map(|entry| entry.tile),
                    Some(Tile::Player { .. })
                )
            });

            match enemy_around.copied() {
                Some((direction, _)) if free_neighbours.len() == 0 => {
                    Some(Action::Attack { direction })
                }
                Some((direction, _)) => {
                    let move_direction = match direction {
                        Direction::Right => Direction::Left,
                        Direction::Up => Direction::Down,
                        Direction::Left => Direction::Right,
                        Direction::Down => Direction::Up,
                    };

                    run_away = Some(move_direction);
                    Some(Action::Place { direction })
                }
                _ => {
                    let distance_and_direction = if player_position.x == position.x {
                        Some((
                            player_position.y.abs_diff(position.y),
                            if player_position.y > position.y {
                                Direction::Up
                            } else {
                                Direction::Down
                            },
                        ))
                    } else if player_position.y == position.y {
                        Some((
                            player_position.x.abs_diff(position.x),
                            if player_position.x > position.x {
                                Direction::Right
                            } else {
                                Direction::Left
                            },
                        ))
                    } else {
                        None
                    };

                    match distance_and_direction {
                        Some((distance, direction))
                            if distance <= gun_level && gun_level > *enemy_gun_level =>
                        {
                            Some(Action::Attack { direction })
                        }
                        _ => None,
                    }
                }
            }
        } else {
            None
        };

        let (moves, new_position, optional_mining_direction) = match run_away {
            Some(direction) if self.cage_step != 4 => (
                Moves {
                    mvs: ::std::iter::once(Some(direction))
                        .cycle()
                        .take(self.player_stats.wheel_level as usize)
                        .chain(
                            ::std::iter::once(None)
                                .take(3 - self.player_stats.wheel_level as usize),
                        )
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                },
                ShittyPosition {
                    x: self.map.player_position.x
                        + match direction {
                            Direction::Right => self.player_stats.wheel_level as i8,
                            Direction::Left => -(self.player_stats.wheel_level as i8),
                            _ => 0,
                        },
                    y: self.map.player_position.y
                        + match direction {
                            Direction::Down => self.player_stats.wheel_level as i8,
                            Direction::Up => -(self.player_stats.wheel_level as i8),
                            _ => 0,
                        },
                },
                self.moves().2,
            ),
            _ => self.moves(),
        };
        let neighbour = self
            .map
            .find_neighbour(new_position, Tile::Osmium)
            .or_else(|| self.map.find_neighbour(new_position, Tile::Iron))
            .or_else(|| self.map.find_neighbour(new_position, Tile::Stone));

        let action = if self.final_phase_entryway.is_some() {
            let center = self.map.center();
            let (direction_from_center, closest) = self
                .map
                .neighbours(center)
                .iter()
                .min_by_key(|(_, position)| self.map.distance_to(*position))
                .copied()
                .unwrap();

            let third_spot = center.add_direction(!direction_from_center);
            if closest == self.map.player_position {
                let side_cw = closest.add_direction(direction_from_center.clockwise());
                let side_ccw = closest.add_direction(direction_from_center.counterclockwise());
                if self.map.tile_at(center).unwrap().tile != Tile::Air {
                    Some(Action::Mine {
                        direction: !direction_from_center,
                    })
                } else if !self.map.tile_at(side_cw).unwrap().tile.is_stone() {
                    Some(Action::Place {
                        direction: direction_from_center.clockwise(),
                    })
                } else if !self.map.tile_at(side_ccw).unwrap().tile.is_stone() {
                    Some(Action::Place {
                        direction: direction_from_center.counterclockwise(),
                    })
                } else {
                    // TODO: USE SCAN HERE IF WE HAVE THE ANTENA
                    None
                }
            } else if self.map.player_position == center {
                let side_cw = center.add_direction(direction_from_center.clockwise());
                let side_ccw = center.add_direction(direction_from_center.counterclockwise());
                if self.map.tile_at(third_spot).unwrap().tile != Tile::Air {
                    Some(Action::Mine {
                        direction: !direction_from_center,
                    })
                } else if self.map.tile_at(side_cw).unwrap().tile != Tile::Air {
                    Some(Action::Mine {
                        direction: direction_from_center.clockwise(),
                    })
                } else if self.map.tile_at(side_ccw).unwrap().tile != Tile::Air {
                    Some(Action::Mine {
                        direction: direction_from_center.counterclockwise(),
                    })
                } else {
                    // TODO: USE SCAN HERE IF WE HAVE THE ANTENA
                    None
                }
            } else {
                let neighbours = self.map.neighbours(center);

                if let Some((direction, _)) = neighbours
                    .iter()
                    .copied()
                    .find(|(d, p)| Some(p.add_direction(*d)) == self.opponent_exact_position())
                {
                    Some(Action::Attack { direction })
                } else {
                    None
                }
            }
        } else if let Some(_) = attack {
            attack
        } else if let Some(direction) = optional_mining_direction {
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

        // println!("{}", &self.map);
        // println!("{:?}", &self.map.dimensions);

        GameOutput {
            moves: Some(moves),
            action,
            upgrade,
        }
    }
}
