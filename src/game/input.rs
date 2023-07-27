use super::output::{Direction, Moves};
use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: u8,
    pub height: u8,
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
pub struct ParentInfo {
    // The move to make to reach this position from the parent
    direction: Direction,
    // The location of the parent
    location: ShittyPosition,
    // Whether this block requires to be mined
    requires_mining: bool,

    // if we have n moves per turn, this represents the index of the
    // move planned to reach this tile.
    pub per_turn_move_index: usize,
}

impl ParentInfo {
    pub fn new(
        direction: Direction,
        parent_location: ShittyPosition,
        requires_mining: bool,
        per_turn_move_index: usize,
    ) -> Self {
        Self {
            direction,
            location: parent_location,
            requires_mining,
            per_turn_move_index,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapEntry {
    pub distance: usize,
    pub parent: Option<ParentInfo>,
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
    pub player_position: ShittyPosition,
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.dimensions.width {
            for j in 0..self.dimensions.height {
                let position = ShittyPosition::new(i as i8, j as i8);
                let char = match self.tile_at(position).unwrap().tile {
                    _ if self.player_position == position => '*',
                    Tile::Acid => 'A',
                    Tile::Iron => 'I',
                    Tile::Unknown => '?',
                    Tile::Osmium => 'O',
                    Tile::Stone => 'S',
                    Tile::Cobblestone => 'C',
                    Tile::Air => ' ',
                    Tile::Bedrock => 'B',
                    _ => '_',
                };
                write!(f, "{char}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Map {
    pub fn merge_with(&mut self, other: &Map, wheel_level: usize) {
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
        self.floodfill(wheel_level);
    }

    pub fn set_acid_level(&mut self, level: usize) {
        let level = level as i8;
        let width = self.dimensions.width as i8;
        let height = self.dimensions.height as i8;
        for i in 0..level {
            for j in i..(height - i) {
                self.set_tile_at(ShittyPosition::new(i, j), Tile::Acid);
                self.set_tile_at(ShittyPosition::new(width - i - 1, j), Tile::Acid);
            }

            for j in i..(width - i) {
                self.set_tile_at(ShittyPosition::new(j, i), Tile::Acid);
                self.set_tile_at(ShittyPosition::new(j, height - i - 1), Tile::Acid);
            }
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

    pub fn closest_tile(&self, target: Tile) -> Option<ShittyPosition> {
        self.find_tiles(target)
            .iter()
            .min_by_key(|position| self.tile_at(**position).unwrap().distance)
            .copied()
    }

    /// Returns:
    /// - The moves to perform
    /// - The location reached by performing said moves
    /// - An optional mining operation which might be required at the end of the turn
    pub fn move_towards(
        &self,
        to: ShittyPosition,
        wheel_level: usize,
    ) -> (Moves, ShittyPosition, Option<Direction>) {
        let mut location = to;
        // Pairs containing `per_turn_move_index` and `ParentInfo`.
        let mut moves: VecDeque<ParentInfo> = VecDeque::new();
        let mut mininig_direction = None;

        while location != self.player_position {
            let entry = self.tile_at(location).unwrap();
            let parent = entry.parent.unwrap();

            if let Some(first_move) = moves.get(0).copied() {
                if first_move.per_turn_move_index == 0 {
                    moves.clear();
                    mininig_direction = if first_move.requires_mining {
                        Some(first_move.direction)
                    } else {
                        None
                    };
                };
            }

            moves.push_front(ParentInfo::new(
                parent.direction,
                location,
                parent.requires_mining,
                parent.per_turn_move_index,
            ));

            location = parent.location;

            if moves.len() > wheel_level {
                moves.pop_back();
            }
        }

        let mut final_moves = [None; 3];

        for i in 0..moves.len() {
            final_moves[i] = Some(moves[i].direction);
        }

        (
            Moves::new(final_moves),
            moves[moves.len() - 1].location,
            mininig_direction,
        )
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

    pub fn floodfill(&mut self, wheel_level: usize) {
        let mut queue: HashSet<ShittyPosition> = HashSet::new();

        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height {
                let position = ShittyPosition::new(x as i8, y as i8);
                queue.insert(position);
                let entry = self.tile_at_mut(position).unwrap();
                entry.distance = usize::MAX / 2;
                entry.parent = None;
            }
        }

        let source_entry = self.tile_at_mut(self.player_position).unwrap();
        source_entry.distance = 0;

        while queue.len() > 0 {
            let min_position = queue
                .iter()
                .min_by_key(|e| self.tile_at(**e).unwrap().distance)
                .copied()
                .unwrap();

            queue.remove(&min_position);

            let min_entry = self.tile_at(min_position).unwrap();

            // TODO: handle multiple wheels
            for (direction, neighbour) in self.neighbours(min_position) {
                if queue.contains(&neighbour) {
                    let entry = self.tile_at_mut(neighbour).unwrap();
                    let per_turn_move_index = min_entry
                        .parent
                        .map_or(0, |p| (p.per_turn_move_index + 1) % wheel_level);

                    let first_move = min_entry.parent.is_none();

                    let tile_info = match entry.tile {
                        Tile::Osmium => Some((2, true)),
                        Tile::Iron => Some((4, true)),
                        Tile::Stone | Tile::Cobblestone => Some((8, true)),
                        Tile::Unknown => Some((6, false)),
                        Tile::Air | Tile::Base => Some((5, false)),
                        _ => None,
                    };

                    if let Some((weight, requires_mining)) = tile_info {
                        let alt = min_entry.distance
                            + if requires_mining && first_move {
                                1000
                            } else if requires_mining && per_turn_move_index == 0 {
                                1
                            } else {
                                weight
                            };

                        if alt < entry.distance {
                            entry.distance = alt;
                            entry.parent = Some(ParentInfo::new(
                                direction,
                                min_position,
                                requires_mining,
                                if requires_mining {
                                    0
                                } else {
                                    per_turn_move_index
                                },
                            ));
                        }
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

        let map = Map {
            dimensions,
            tiles,
            player_position,
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
