use crate::game::output::Direction;

use super::output::{GameOutput, Moves};

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    width: u8,
    height: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tile {
    Air,
    Stone,
    Cobblestone,
    Bedrock,
    Iron,
    Osmium,
    Base,
    Acid,

    Player {
        id: u8,
    },

    #[default]
    Unknown,
}

impl Tile {
    pub fn is_ore(self) -> bool {
        match self {
            Tile::Osmium | Tile::Iron => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Map {
    pub dimensions: Dimensions,
    pub tiles: Box<[Box<[Tile]>]>,
}

impl Map {
    pub fn merge_with(&mut self, other: &Map) {
        for i in 0..self.dimensions.width.into() {
            for j in 0..self.dimensions.height.into() {
                if self.tiles[i][j] != other.tiles[i][j] && other.tiles[i][j] != Tile::Unknown {
                    self.tiles[i][j] = other.tiles[i][j]
                }
            }
        }
    }

    pub fn distance_to(&self, position: (usize, usize)) -> usize {
        todo!()
    }

    pub fn find_tiles(&self, target: Tile) -> Vec<(usize, usize)> {
        self.tiles
            .iter()
            .enumerate()
            .flat_map(|(x, array)| {
                array
                    .iter()
                    .enumerate()
                    .filter_map(|(y, tile)| if *tile == target { Some((x, y)) } else { None })
                    .collect::<Vec<(usize, usize)>>()
            })
            .collect()
    }

    pub fn closest_tile(&self, target: Tile) -> Option<(usize, usize)> {
        self.find_tiles(target)
            .iter()
            .min_by_key(|position| self.distance_to(**position))
            .copied()
    }

    pub fn move_towards(&self, position: (usize, usize)) -> (Option<Moves>, (usize, usize)) {
        todo!()
    }

    pub fn tile_at(&self, position: (usize, usize)) -> Option<Tile> {
        self.tiles
            .get(position.0)
            .and_then(|array| array.get(position.1))
            .map(|b| *b)
    }

    pub fn neighbours(&self, position: (usize, usize)) -> Vec<(Direction, (usize, usize))> {
        vec![
            (Direction::Right, (position.0 + 1, position.1)),
            (Direction::Left, (position.0 - 1, position.1)),
            (Direction::Down, (position.0, position.1 + 1)),
            (Direction::Up, (position.0, position.1 - 1)),
        ]
    }

    pub fn find_neighbour(&self, position: (usize, usize), target: Tile) -> Option<(Direction, (usize, usize))> {
        self.neighbours(position)
            .iter()
            .filter(|(_, location)| self.tile_at(*location) == Some(target))
            .copied()
            .next()
    }

    pub fn can_upgrade(&self, stats: PlayerStats, position: PlayerPosition) {
            let base = self.closest_tile(Tile::Base).unwrap();

            if !self.player_stats.has_battery && base != self.player_position.as_vec() {
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerPosition {
    pub x: u8,
    pub y: u8,
}

impl PlayerPosition {
    pub fn as_vec(self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

#[derive(Debug)]
pub struct PlayerStats {
    pub hit_points: u8,

    pub drill_level: u8,
    pub gun_level: u8,
    pub wheel_level: u8,
    pub camera_level: u8,

    pub has_antenna: bool,
    pub has_battery: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerInventory {
    pub stone: u16,
    pub iron: u16,
    pub osmium: u16,
}

impl PlayerInventory {
    pub fn can_afford(self, other: Self) -> bool {
        self.osmium >= other.osmium && self.iron >= other.iron
    }
}

#[derive(Debug)]
pub struct GameInput {
    pub map: Map,
    pub player_position: PlayerPosition,
    pub player_stats: PlayerStats,
    pub player_inventory: PlayerInventory,
}

impl TryFrom<&str> for GameInput {
    type Error = !;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let mut lines = input.lines();

        let dimensions = {
            let (width, height) = lines.next().unwrap().split_once(' ').unwrap();

            Dimensions {
                width: width.parse().unwrap(),
                height: height.parse().unwrap(),
            }
        };

        let tiles = {
            let mut tiles = vec![
                vec![Tile::Unknown; dimensions.height.into()].into_boxed_slice();
                dimensions.width.into()
            ]
            .into_boxed_slice();

            for j in 0..dimensions.height.into() {
                let row = lines.next().unwrap();

                for (i, tile) in row
                    .split_ascii_whitespace()
                    .filter_map(|tile| (tile != " ").then_some(tile.chars().next().unwrap()))
                    .enumerate()
                {
                    let tile = match tile {
                        '.' => Tile::Air,
                        'X' => Tile::Stone,
                        'A' => Tile::Cobblestone,
                        'B' => Tile::Bedrock,
                        'C' => Tile::Iron,
                        'D' => Tile::Osmium,
                        'E' => Tile::Base,
                        'F' => Tile::Acid,

                        '0'..='9' => Tile::Player {
                            id: u8::try_from(tile).unwrap() - 48,
                        },

                        '?' => Tile::Unknown,
                        _ => panic!("Unknown tile {tile}"),
                    };

                    tiles[i][j] = tile
                }
            }

            tiles
        };

        let map = Map { dimensions, tiles };

        let player_position = {
            let (x, y) = lines.next().unwrap().split_once(' ').unwrap();

            PlayerPosition {
                x: x.parse().unwrap(),
                y: y.parse().unwrap(),
            }
        };

        let player_stats = {
            let mut stats = lines.next().unwrap().split_ascii_whitespace();

            let hit_points = stats.next().unwrap().parse().unwrap();

            let drill_level = stats.next().unwrap().parse().unwrap();
            let gun_level = stats.next().unwrap().parse().unwrap();
            let wheel_level = stats.next().unwrap().parse().unwrap();
            let camera_level = stats.next().unwrap().parse().unwrap();

            let has_antenna = stats.next().unwrap().parse::<u8>().unwrap() != 0;
            let has_battery = stats.next().unwrap().parse::<u8>().unwrap() != 0;

            PlayerStats {
                hit_points,

                drill_level,
                gun_level,
                wheel_level,
                camera_level,

                has_antenna,
                has_battery,
            }
        };

        let player_inventory = {
            let mut inventory = lines.next().unwrap().split_ascii_whitespace();

            let stone = inventory.next().unwrap().parse().unwrap();
            let iron = inventory.next().unwrap().parse().unwrap();
            let osmium = inventory.next().unwrap().parse().unwrap();

            PlayerInventory {
                stone,
                iron,
                osmium,
            }
        };

        Ok(GameInput {
            map,
            player_position,
            player_stats,
            player_inventory,
        })
    }
}
