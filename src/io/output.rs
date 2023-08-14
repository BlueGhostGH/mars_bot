#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Output
{
    pub(crate) moves: Option<moves::Moves>,
    pub(crate) action: Option<action::Action>,
    pub(crate) upgrade: Option<upgrade::Upgrade>,
}

pub(crate) fn show(
    Output {
        moves,
        action,
        upgrade,
    }: Output,
) -> String
{
    [
        moves.map(moves::show),
        action.map(action::show),
        upgrade.map(upgrade::show),
    ]
    .into_iter()
    .flatten()
    .intersperse("\n".into())
    .collect()
}

pub(crate) mod direction
{
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    // NOTE: Defined in trigonometric order
    pub(crate) enum Direction
    {
        Right,
        Up,
        Left,
        Down,
    }

    impl Direction
    {
        pub(crate) fn opposite(self) -> Self
        {
            match self {
                Direction::Right => Direction::Left,
                Direction::Up => Direction::Down,
                Direction::Left => Direction::Right,
                Direction::Down => Direction::Up,
            }
        }

        pub(crate) fn clockwise(self) -> Self
        {
            match self {
                Direction::Right => Direction::Down,
                Direction::Up => Direction::Right,
                Direction::Left => Direction::Up,
                Direction::Down => Direction::Left,
            }
        }

        pub(crate) fn counter_clockwise(self) -> Self
        {
            match self {
                Direction::Right => Direction::Up,
                Direction::Up => Direction::Left,
                Direction::Left => Direction::Down,
                Direction::Down => Direction::Right,
            }
        }
    }

    pub(super) fn show(direction: Direction) -> char
    {
        match direction {
            Direction::Right => 'R',
            Direction::Up => 'U',
            Direction::Left => 'L',
            Direction::Down => 'D',
        }
    }
}

pub(crate) mod moves
{
    use crate::io::output::direction;

    #[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Moves
    {
        pub(crate) mvs: [Option<direction::Direction>; 3],
    }

    pub(super) fn show(moves: Moves) -> String
    {
        moves
            .mvs
            .into_iter()
            .flatten()
            .map(direction::show)
            .intersperse(' ')
            .collect()
    }
}

pub(crate) mod action
{
    use crate::io::output::direction;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum Action
    {
        #[allow(dead_code)]
        Attack
        {
            direction: direction::Direction
        },
        #[allow(dead_code)]
        Scan
        {
            direction: direction::Direction
        },
        Mine
        {
            direction: direction::Direction
        },
        #[allow(dead_code)]
        Place
        {
            direction: direction::Direction
        },
    }

    pub(super) fn show(action: Action) -> String
    {
        let (action, direction) = match action {
            Action::Attack { direction } => ('A', direction),
            Action::Scan { direction } => ('S', direction),
            Action::Mine { direction } => ('M', direction),
            Action::Place { direction } => ('P', direction),
        };

        [action, ' ', direction::show(direction)]
            .into_iter()
            .collect()
    }
}

pub(crate) mod upgrade
{
    use crate::{constants::upgrade, io::input::player};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum Upgrade
    {
        Sight,
        Attack,
        Drill,
        Movement,

        Radar,
        Battery,

        Heal,
    }

    impl Upgrade
    {
        #[allow(dead_code)]
        pub(crate) fn cost(
            &self,
            player::stats::Stats {
                drl_level: drill_level,
                gun_level,
                whl_level: wheel_level,
                cmr_level: camera_level,
                ..
            }: &player::stats::Stats,
        ) -> Option<Cost>
        {
            match self {
                Upgrade::Sight => upgrade::SIGHT_COSTS.get(*camera_level as usize).copied(),
                Upgrade::Attack => upgrade::ATTACK_COSTS.get(*gun_level as usize).copied(),
                Upgrade::Drill => upgrade::DRILL_COSTS.get(*drill_level as usize).copied(),
                Upgrade::Movement => upgrade::MOVEMENT_COSTS.get(*wheel_level as usize).copied(),

                Upgrade::Radar => Some(upgrade::RADAR_COST),
                Upgrade::Battery => Some(upgrade::BATTERY_COST),

                Upgrade::Heal => Some(upgrade::HEAL_COST),
            }
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) struct Cost
    {
        pub(crate) iron: u16,
        pub(crate) osmium: u16,
    }

    pub(super) fn show(upgrade: Upgrade) -> String
    {
        let upgrade = match upgrade {
            Upgrade::Sight => 'S',
            Upgrade::Attack => 'A',
            Upgrade::Drill => 'D',
            Upgrade::Movement => 'M',

            Upgrade::Radar => 'R',
            Upgrade::Battery => 'B',

            Upgrade::Heal => 'H',
        };

        ['B', ' ', upgrade].into_iter().collect()
    }
}
