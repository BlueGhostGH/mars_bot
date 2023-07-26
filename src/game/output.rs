use super::input::{PlayerInventory, PlayerStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
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

#[derive(Debug, Clone, Copy)]
pub enum Moves {
    One {
        first: Direction,
    },
    Two {
        first: Direction,
        second: Direction,
    },
    Three {
        first: Direction,
        second: Direction,
        third: Direction,
    },
}

#[derive(Debug)]
pub enum Action {
    Attack { direction: Direction },
    Scan { direction: Direction },
    Mine { direction: Direction },
    Place { direction: Direction },
}

#[derive(Debug)]
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
    pub fn cost(self, stats: PlayerStats) -> PlayerInventory {
        todo!()
    }
}

#[derive(Debug)]
pub struct GameOutput {
    pub(crate) moves: Option<Moves>,
    pub(crate) action: Option<Action>,
    pub(crate) upgrade: Option<Upgrade>,
}

impl Into<String> for GameOutput {
    fn into(self) -> String {
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
