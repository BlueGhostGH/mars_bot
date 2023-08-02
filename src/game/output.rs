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
    .filter_map(::core::convert::identity)
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
    use crate::game::output::direction;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    use crate::game::output::direction;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) enum Action
    {
        Attack
        {
            direction: direction::Direction
        },
        Scan
        {
            direction: direction::Direction
        },
        Mine
        {
            direction: direction::Direction
        },
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
