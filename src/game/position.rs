use crate::game::input::dimensions;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Position
{
    pub(crate) x: isize,
    pub(crate) y: isize,
}

impl Position
{
    pub(crate) fn is_within_bounds(self, dimensions: dimensions::Dimensions) -> bool
    {
        // NOTE: the casts are valid as we'd short-circuit
        // before them if our values were negative
        0 <= self.x
            && (self.x as usize) < dimensions.width
            && 0 <= self.y
            && (self.y as usize) < dimensions.height
    }

    pub(crate) fn from_linear(i: usize, width: usize) -> Self
    {
        Position {
            x: (i % width) as _,
            y: (i / width) as _,
        }
    }

    pub(crate) fn to_linear(self, width: usize) -> usize
    {
        (self.y * width as isize + self.x) as _
    }
}
