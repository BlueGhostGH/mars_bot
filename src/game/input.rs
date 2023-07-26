use super::output::{Direction, Moves};
use std::collections::{HashMap, HashSet, VecDeque};

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
            for (_, w) in self.find_neighbour(v, Tile::Air) {
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

    pub fn closest_tile(&self, from: ShittyPosition, target: Tile) -> Option<ShittyPosition> {
        self.find_tiles(target)
            .iter()
            .min_by_key(|position| self.distance_from_to(from, **position))
            .copied()
    }

    pub fn move_towards(
        &self,
        from: ShittyPosition,
        to: ShittyPosition,
        wheel_level: u8,
    ) -> (Option<Moves>, ShittyPosition) {
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
            for w in self.find_neighbour(v, Tile::Air) {
                if explored.get(&w.1) == None {
                    explored.insert(w.1);
                    parents.insert(w.1, (v, w.0));
                    queue.push_back(w.1);
                }
            }
        }

        if found {
            let mut curr = goal;
            let mut dist = 0;

            let mut moves = [None; 3];

            while curr != root {
                moves.rotate_right(1);
                let (curr, dir) = *parents.get(&curr).unwrap();
                moves[0] = Some(dir);
            }

            if moves == [None; 3] {
                return (None, from);
            }

            fn direction_to_position(direction: Direction) -> ShittyPosition {
                match direction {
                    Direction::Right => ShittyPosition { x: 1, y: 0 },
                    Direction::Up => ShittyPosition { x: 0, y: 0 + 1 },
                    Direction::Left => ShittyPosition { x: 0 - 1, y: 0 },
                    Direction::Down => ShittyPosition { x: 0, y: 0 - 1 },
                }
            }

            match wheel_level {
                1 => (
                    Some(Moves::One {
                        first: moves[0].unwrap(),
                    }),
                    ShittyPosition {
                        x: from.x + direction_to_position(moves[0].unwrap()).x,
                        y: from.y + direction_to_position(moves[0].unwrap()).y,
                    },
                ),
                2 => (
                    Some(Moves::Two {
                        first: moves[0].unwrap(),
                        second: moves[1].unwrap(),
                    }),
                    ShittyPosition {
                        x: from.x
                            + direction_to_position(moves[0].unwrap()).x
                            + direction_to_position(moves[1].unwrap()).x,
                        y: from.y
                            + direction_to_position(moves[0].unwrap()).y
                            + direction_to_position(moves[1].unwrap()).y,
                    },
                ),
                3 => (
                    Some(Moves::Three {
                        first: moves[0].unwrap(),
                        second: moves[1].unwrap(),
                        third: moves[2].unwrap(),
                    }),
                    ShittyPosition {
                        x: from.x
                            + direction_to_position(moves[0].unwrap()).x
                            + direction_to_position(moves[1].unwrap()).x
                            + direction_to_position(moves[2].unwrap()).x,
                        y: from.y
                            + from.y
                            + direction_to_position(moves[0].unwrap()).y
                            + direction_to_position(moves[1].unwrap()).y
                            + direction_to_position(moves[2].unwrap()).y,
                    },
                ),
                _ => unreachable!(),
            }
        } else {
            (None, from)
        }
    }

    pub fn tile_at(&self, position: ShittyPosition) -> Option<Tile> {
        self.tiles
            .get(position.x as usize)
            .and_then(|array| array.get(position.y as usize))
            .map(|b| *b)
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
