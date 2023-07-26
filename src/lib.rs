#![feature(byte_slice_trim_ascii, never_type, iter_intersperse)]
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    explicit_outlives_requirements,
    let_underscore_drop,
    missing_copy_implementations,
    missing_debug_implementations,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    unreachable_pub,
    unused_results,
    variant_size_differences
)]
#![allow(clippy::new_without_default)]

#[derive(Debug)]
struct Dimensions
{
    width: u8,
    height: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Tile
{
    Air,
    Stone,
    Cobblestone,
    Bedrock,
    Iron,
    Osmium,
    Base,
    Acid,

    Player
    {
        id: u8,
    },

    #[default]
    Unknown,
}

#[derive(Debug)]
struct Map
{
    tiles: Box<[Box<[Tile]>]>,
}

#[derive(Debug)]
struct PlayerPosition
{
    x: u8,
    y: u8,
}

#[derive(Debug)]
struct PlayerStats
{
    hit_points: u8,

    drill_level: u8,
    gun_level: u8,
    wheel_level: u8,
    camera_level: u8,

    has_antenna: bool,
    has_battery: bool,
}

#[derive(Debug)]
struct PlayerInventory
{
    stone: u16,
    iron: u16,
    osmium: u16,
}

#[derive(Debug)]
pub struct GameInput
{
    dimensions: Dimensions,
    map: Map,
    player_position: PlayerPosition,
    player_stats: PlayerStats,
    player_inventory: PlayerInventory,
}

impl TryFrom<&str> for GameInput
{
    type Error = !;

    fn try_from(input: &str) -> Result<Self, Self::Error>
    {
        let mut lines = input.lines();

        let dimensions = {
            let (width, height) = lines.next().unwrap().split_once(' ').unwrap();

            Dimensions {
                width: width.parse().unwrap(),
                height: height.parse().unwrap(),
            }
        };

        let map = {
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

            Map { tiles }
        };

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
            dimensions,
            map,
            player_position,
            player_stats,
            player_inventory,
        })
    }
}

#[derive(Debug)]
enum Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction
{
    Right,
    Up,
    Left,
    Down,
}

impl Into<char> for Direction
{
    fn into(self) -> char
    {
        match self {
            Direction::Right => 'R',
            Direction::Up => 'U',
            Direction::Left => 'L',
            Direction::Down => 'D',
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Moves
{
    One
    {
        first: Direction
    },
    Two
    {
        first: Direction, second: Direction
    },
    Three
    {
        first: Direction,
        second: Direction,
        third: Direction,
    },
}

#[derive(Debug)]
enum Action
{
    Attack
    {
        direction: Direction
    },
    Scan
    {
        direction: Direction
    },
    Mine
    {
        direction: Direction
    },
    Place
    {
        direction: Direction
    },
}

#[derive(Debug)]
enum Upgrade
{
    Sight,
    Attack,
    Drill,
    Movement,

    Radar,
    Battery,

    Heal,
}

#[derive(Debug)]
pub struct GameOutput
{
    moves: Option<Moves>,
    action: Option<Action>,
    upgrade: Option<Upgrade>,
}

impl Into<String> for GameOutput
{
    fn into(self) -> String
    {
        let moves = self.moves.map(|moves| {
            match moves {
                Moves::One { first } => ::std::iter::once(first).collect::<Vec<_>>(),
                Moves::Two { first, second } => ::std::iter::once(first)
                    .chain(::std::iter::once(second))
                    .collect::<Vec<_>>(),
                Moves::Three {
                    first,
                    second,
                    third,
                } => ::std::iter::once(first)
                    .chain(::std::iter::once(second))
                    .chain(::std::iter::once(third))
                    .collect::<Vec<_>>(),
            }
            .into_iter()
            .map(|direction| direction.into())
            .intersperse(' ')
            .collect::<String>()
        });

        let action = self.action.map(|action| {
            let (action, direction) = match action {
                Action::Attack { direction } => ('A', direction),
                Action::Scan { direction } => ('S', direction),
                Action::Mine { direction } => ('M', direction),
                Action::Place { direction } => ('P', direction),
            };

            format!("{action} {}", <Direction as Into<char>>::into(direction))
        });

        let upgrade = self.upgrade.map(|upgrade| {
            let upgrade = match upgrade {
                Upgrade::Sight => 'S',
                Upgrade::Attack => 'A',
                Upgrade::Drill => 'D',
                Upgrade::Movement => 'M',

                Upgrade::Radar => 'R',
                Upgrade::Battery => 'B',

                Upgrade::Heal => 'H',
            };

            format!("B {upgrade}")
        });

        [moves, action, upgrade]
            .into_iter()
            .flatten()
            .intersperse(String::from("\n"))
            .collect::<String>()
    }
}

pub fn magic(input: GameInput) -> GameOutput
{
    GameOutput {
        moves: Some(Moves::Two {
            first: Direction::Up,
            second: Direction::Right,
        }),
        action: Some(Action::Attack {
            direction: Direction::Right,
        }),
        upgrade: Some(Upgrade::Heal),
    }
}
