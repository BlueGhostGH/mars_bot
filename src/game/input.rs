use super::output::{Direction, Moves};
use std::{collections::HashSet, todo};

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

#[derive(Debug, Clone, Copy)]
pub struct MapEntry {
    pub distance: usize,
    pub parent: Option<ShittyPosition>,
    pub tile: Tile,
}

impl MapEntry {
    pub fn new(tile: Tile) -> Self {
        let mut result = Self::default();
        result.tile = tile;
        result
    }
}

impl Default for MapEntry {
    fn default() -> Self {
        Self {
            distance: usize::MAX,
            parent: None,
            tile: Tile::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Map {
    pub dimensions: Dimensions,
    pub tiles: Box<[Box<[MapEntry]>]>,
    pub player_position: ShittyPosition
}

impl Map {
    pub fn merge_with(&mut self, other: &Map, player_position: ShittyPosition) {
        for i in 0..self.dimensions.width.into() {
            for j in 0..self.dimensions.height.into() {
                if self.tiles[i][j].tile != other.tiles[i][j].tile
                    && other.tiles[i][j].tile != Tile::Unknown
                    && other.tiles[i][j].tile != Tile::Base
                {
                    self.tiles[i][j].tile = other.tiles[i][j].tile
                }
            }
        }

        self.player_position = other.player_position;
        self.floodfill(player_position);
    }

    pub fn find_tiles(&self, target: Tile) -> Vec<ShittyPosition> {
        self.tiles
            .iter()
            .enumerate()
            .flat_map(|(x, array)| {
                array
                    .iter()
                    .enumerate()
                    .filter_map(|(y, entry)| {
                        if entry.tile == target {
                            Some(ShittyPosition {
                                x: x as i8,
                                y: y as i8,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<ShittyPosition>>()
            })
            .collect()
    }

    pub fn tile_at_mut(&mut self, position: ShittyPosition) -> Option<&mut MapEntry> {
        self.tiles
            .get_mut(position.x as usize)
            .and_then(|array| array.get_mut(position.y as usize))
    }

    pub fn closest_tile(
        &self,
        target: Tile,
    ) -> Option<ShittyPosition> {
        self.find_tiles(target)
            .iter()
            .min_by_key(|position| self.tile_at(**position).unwrap().distance)
            .copied()
    }

    pub fn move_towards(
        &self,
        to: ShittyPosition,
        wheel_level: u8,
    ) -> (Option<Moves>, ShittyPosition) {
        let mut location = to;
        let mut moves  = [None ;3 ];
        while entry != self.player_position {
            moves.
        }
    }

    pub fn tile_at(&self, position: ShittyPosition) -> Option<MapEntry> {
        self.tiles
            .get(position.x as usize)
            .and_then(|array| array.get(position.y as usize))
            .map(|b| *b)
    }

    pub fn set_tile_at(&mut self, position: ShittyPosition, tile: Tile) {
        self.tiles[position.x as usize][position.y as usize].tile = tile;
    }

    pub fn neighbours(&self, position: ShittyPosition) -> [(Direction, ShittyPosition); 4] {
        [
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

    pub fn floodfill(&mut self, from: ShittyPosition) {
        let mut queue: HashSet<ShittyPosition> = HashSet::new();

        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height {
                queue.insert(ShittyPosition::new(x as i8, y as i8));
            }
        }

        while queue.len() > 0 {
            let min_position = queue
                .iter()
                .min_by_key(|e| self.tile_at(**e).unwrap().distance)
                .copied()
                .unwrap();

            queue.remove(&min_position);
            let dist_to_min = self.tile_at(min_position).unwrap().distance;

            // TODO: handle multiple wheels
            for (_, neighbour) in self.neighbours(min_position) {
                if queue.contains(&neighbour) {
                    let entry = self.tile_at_mut(neighbour).unwrap();
                    let weight = match entry.tile {
                        Tile::Osmium => 1,
                        Tile::Iron => 4,
                        Tile::Stone | Tile::Cobblestone => 8,
                        Tile::Unknown => 6,
                        Tile::Bedrock => 1000,
                        Tile::Acid => 1000,
                        _ => 5,
                    };

                    let alt = dist_to_min + weight;
                    if alt < entry.distance {
                        entry.distance = alt;
                        entry.parent = Some(min_position);
                    }
                };
            }
        }
    }

    pub fn find_neighbours(
        &self,
        position: ShittyPosition,
        target: Tile,
    ) -> Vec<(Direction, ShittyPosition)> {
        self.neighbours(position)
            .iter()
            .filter(|(_, location)| self.tile_at(*location).map(|t| t.tile) == Some(target))
            .copied()
            .collect()
    }

    pub fn find_neighbour(
        &self,
        position: ShittyPosition,
        target: Tile,
    ) -> Option<(Direction, ShittyPosition)> {
        self.find_neighbours(position, target).get(0).copied()
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
    pub x: i8,
    pub y: i8,
}

impl ShittyPosition {
    pub fn new(x: i8, y: i8) -> Self {
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
    pub fn new(iron: u16, osmium: u16) -> Self {
        Self { iron, osmium }
    }
}

impl PlayerInventory {
    pub fn can_afford(self, other: UpgradeCost) -> bool {
        self.osmium >= other.osmium && self.iron >= other.iron
    }
}

#[derive(Debug)]
pub struct GameInput {
    pub map: Map,
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
                vec![MapEntry::default(); dimensions.height.into()]
                    .into_boxed_slice();
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

                    tiles[i][j] = MapEntry::new(tile)
                }
            }

            tiles
        };


        let player_position = {
            let (x, y) = lines.next().unwrap().split_once(' ').unwrap();

            ShittyPosition {
                x: x.parse().unwrap(),
                y: y.parse().unwrap(),
            }
        };

        let map = Map { dimensions, tiles, player_position };

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
            player_stats,
            player_inventory,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{
        input::ShittyPosition,
        output::{Direction, Moves},
    };

    use super::{Dimensions, Map, Tile};

    // #[test]
    // fn move_towards_works() {
    //     let map = Map {
    //         dimensions: Dimensions {
    //             width: 4,
    //             height: 4,
    //         },
    //         tiles: vec![
    //             vec![Tile::Player { id: 0 }, Tile::Air, Tile::Air, Tile::Air].into_boxed_slice(),
    //             vec![Tile::Air, Tile::Stone, Tile::Air, Tile::Air].into_boxed_slice(),
    //             vec![Tile::Air, Tile::Air, Tile::Stone, Tile::Air].into_boxed_slice(),
    //             vec![Tile::Air, Tile::Air, Tile::Air, Tile::Iron].into_boxed_slice(),
    //         ]
    //         .into_boxed_slice(),
    //     };
    //
    //     let (moves, final_position) = map.move_towards(
    //         super::ShittyPosition { x: 0, y: 0 },
    //         super::ShittyPosition { x: 3, y: 3 },
    //         1,
    //     );
    //
    //     assert!(match moves {
    //         Some(Moves::One {
    //             first: Direction::Right | Direction::Down,
    //         }) => matches!(
    //             final_position,
    //             ShittyPosition { x: 1, y: 0 } | ShittyPosition { x: 0, y: 1 }
    //         ),
    //         _ => false,
    //     })
    // }
}
