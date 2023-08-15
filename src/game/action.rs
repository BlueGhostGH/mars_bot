use crate::game;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Action
{
    #[allow(dead_code)]
    Attack
    {
        direction: game::Direction
    },
    #[allow(dead_code)]
    Scan
    {
        direction: game::Direction
    },
    Mine
    {
        direction: game::Direction
    },
    #[allow(dead_code)]
    Place
    {
        direction: game::Direction
    },
}
