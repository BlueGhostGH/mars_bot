use std::{ops::Not, unreachable};

use super::input::{PlayerInventory, PlayerStats, UpgradeCost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    pub const DIRECTIONS: [Direction; 4] = [
        Direction::Left,
        Direction::Up,
        Direction::Right,
        Direction::Down,
    ];

    pub fn clockwise(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    pub fn counterclockwise(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Into<char> for Direction {
    fn into(self) -> char {
        match self {
            Direction::Right => 'R',
            Direction::Up => 'U',
            Direction::Left => 'L',
            Direction::Down => 'D',
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Moves {
    pub mvs: [Option<Direction>; 3],
}

impl Moves {
    pub fn new(mvs: [Option<Direction>; 3]) -> Self {
        Self { mvs }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Attack { direction: Direction },
    Scan { direction: Direction },
    Mine { direction: Direction },
    Place { direction: Direction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upgrade {
    Sight,
    Attack,
    Drill,
    Movement,

    Radar,
    Battery,

    Heal,
}

impl Upgrade {
    pub fn cost(self, stats: PlayerStats) -> UpgradeCost {
        match self {
            Self::Heal => UpgradeCost::new(0, 1),
            Self::Battery => UpgradeCost::new(1, 1),
            Self::Radar => UpgradeCost::new(2, 1),
            Self::Attack => match stats.gun_level {
                1 => UpgradeCost::new(3, 0),
                2 => UpgradeCost::new(6, 1),
                _ => unreachable!(),
            },
            Self::Sight => match stats.camera_level {
                1 => UpgradeCost::new(3, 0),
                2 => UpgradeCost::new(6, 1),
                _ => unreachable!(),
            },
            Self::Drill => match stats.drill_level {
                1 => UpgradeCost::new(3, 100),
                2 => UpgradeCost::new(6, 100),
                _ => unreachable!(),
            },
            Self::Movement => match stats.wheel_level {
                1 => UpgradeCost::new(3, 0),
                2 => UpgradeCost::new(6, 3),
                _ => unreachable!(),
            },
        }
    }

    pub const UPGRADE_QUEUE: [Upgrade; 10] = [
        Upgrade::Battery,
        Upgrade::Sight,
        Upgrade::Movement,
        Upgrade::Attack,
        Upgrade::Sight,
        Upgrade::Attack,
        Upgrade::Movement,
        Upgrade::Radar,
        Upgrade::Drill,
        Upgrade::Drill,
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct GameOutput {
    pub(crate) moves: Option<Moves>,
    pub(crate) action: Option<Action>,
    pub(crate) upgrade: Option<Upgrade>,
}

impl Into<String> for GameOutput {
    fn into(self) -> String {
        let moves = self.moves.map(|moves| {
            moves
                .mvs
                .into_iter()
                .flatten()
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
