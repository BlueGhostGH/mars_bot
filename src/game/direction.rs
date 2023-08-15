#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// NOTE: Defined in trigonometric order
pub(crate) enum Direction
{
    Right,
    Up,
    Left,
    Down,
}

pub(crate) const DIRECTIONS: [Direction; 4] = [
    Direction::Right,
    Direction::Up,
    Direction::Left,
    Direction::Down,
];

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
