use std::collections::{HashMap, HashSet, VecDeque};

use super::output::{Direction, GameOutput, Moves};

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

    pub fn distance_from_to(&self, from: ShittyPosition, to: ShittyPosition) -> Option<usize> {
        let root = from;
        let goal = to;
        let mut found = false;

        let mut explored = HashSet::new();
        let mut parents = HashMap::new();

        let mut queue = VecDeque::new();
        explored.insert(from);
        queue.push_back(from);
        while !queue.is_empty() {
            let v = queue.pop_front().unwrap();
            if v == goal {
                found = true;
                break;
            }
            for w in [
                ShittyPosition { x: v.x + 1, y: v.y },
                ShittyPosition { x: v.x, y: v.y + 1 },
                ShittyPosition { x: v.x - 1, y: v.y },
                ShittyPosition { x: v.x, y: v.y - 1 },
            ]
            .into_iter()
            .filter(|&ShittyPosition { x, y }| self.tiles[x as usize][y as usize] == Tile::Air)
            {
                if explored.get(&w) == None {
                    explored.insert(w);
                    parents.insert(w, v);
                    queue.push_back(w);
                }
            }
        }

        if found {
            let mut curr = goal;
            let mut dist = 0;

            while curr != root {
                curr = *parents.get(&curr).unwrap();
                dist += 1;
            }

            Some(dist)
        } else {
            None
        }
    }

    pub fn find_tiles(&self, target: Tile) -> Vec<ShittyPosition> {
        self.tiles
            .iter()
            .enumerate()
            .flat_map(|(x, array)| {
                array
                    .iter()
                    .enumerate()
                    .filter_map(|(y, tile)| {
                        if *tile == target {
                            Some(ShittyPosition {
                                x: x as u8,
                                y: y as u8,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<ShittyPosition>>()
            })
            .collect()
    }

    pub fn closest_tile(&self, target: Tile) -> Option<ShittyPosition> {
        self.find_tiles(target)
            .iter()
            .min_by_key(|position| self.distance_from_to(todo!(), **position))
            .copied()
    }

    pub fn move_towards(&self, position: ShittyPosition) -> (Option<Moves>, ShittyPosition) {
        todo!()
    }

    pub fn tile_at(&self, position: ShittyPosition) -> Option<Tile> {
        self.tiles
            .get(position.x as usize)
            .and_then(|array| array.get(position.y as usize))
            .map(|b| *b)
    }

    pub fn neighbours(&self, position: ShittyPosition) -> Vec<(Direction, ShittyPosition)> {
        vec![
            (
                Direction::Right,
                ShittyPosition::new(position.x + 1, position.y),
            ),
            (
                Direction::Left,
                ShittyPosition::new(position.x - 1, position.y),
            ),
            (
                Direction::Down,
                ShittyPosition::new(position.x, position.y + 1),
            ),
            (
                Direction::Up,
                ShittyPosition::new(position.x, position.y - 1),
            ),
        ]
    }

    pub fn find_neighbour(
        &self,
        position: ShittyPosition,
        target: Tile,
    ) -> Option<(Direction, ShittyPosition)> {
        self.neighbours(position)
            .iter()
            .filter(|(_, location)| self.tile_at(*location) == Some(target))
            .copied()
            .next()
    }
}

// 1  procedure BFS(G, root) is
// 2      let Q be a queue
// 3      label root as explored
// 4      Q.enqueue(root)
// 5      while Q is not empty do
// 6          v := Q.dequeue()
// 7          if v is the goal then
// 8              return v
// 9          for all edges from v to w in G.adjacentEdges(v) do
// 10              if w is not labeled as explored then
// 11                  label w as explored
// 12                  w.parent := v
// 13                  Q.enqueue(w)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShittyPosition {
    pub x: u8,
    pub y: u8,
}

impl ShittyPosition {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct UpgradeCost {
    pub iron: u16,
    pub osmium: u16,
}

impl UpgradeCost {
    pub fn new(iron: u16, osmium: u16) -> Self { Self { iron, osmium } }
}

impl PlayerInventory {
    pub fn can_afford(self, other: UpgradeCost) -> bool {
        self.osmium >= other.osmium && self.iron >= other.iron
    }
}

#[derive(Debug)]
pub struct GameInput {
    pub map: Map,
    pub player_position: ShittyPosition,
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

            ShittyPosition {
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
