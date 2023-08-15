use crate::game;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Output
{
    pub(crate) moves: Option<moves::Moves>,
    pub(crate) action: Option<game::Action>,
    pub(crate) upgrade: Option<game::Upgrade>,
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
    use crate::game;

    pub(super) fn show(direction: game::Direction) -> char
    {
        use game::Direction as D;

        match direction {
            D::Right => 'R',
            D::Up => 'U',
            D::Left => 'L',
            D::Down => 'D',
        }
    }
}

pub(crate) mod moves
{
    use crate::{game, io::output::direction};

    #[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Moves
    {
        pub(crate) mvs: [Option<game::Direction>; 3],
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
    use crate::{game, io::output::direction};

    pub(super) fn show(action: game::Action) -> String
    {
        use game::Action as A;

        let (action, direction) = match action {
            A::Attack { direction } => ('A', direction),
            A::Scan { direction } => ('S', direction),
            A::Mine { direction } => ('M', direction),
            A::Place { direction } => ('P', direction),
        };

        [action, ' ', direction::show(direction)]
            .into_iter()
            .collect()
    }
}

mod upgrade
{
    use crate::game;

    pub(super) fn show(upgrade: game::Upgrade) -> String
    {
        use game::Upgrade as U;

        let upgrade = match upgrade {
            U::Sight => 'S',
            U::Rifle => 'A',
            U::Drill => 'D',
            U::Wheel => 'M',

            U::Antenna => 'R',
            U::Battery => 'B',

            U::Heal => 'H',
        };

        ['B', ' ', upgrade].into_iter().collect()
    }
}
