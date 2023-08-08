use crate::io::input::dimensions;

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

    pub(crate) fn from_linear(i: usize, stride: usize) -> Self
    {
        Position {
            x: (i % stride) as _,
            y: (i / stride) as _,
        }
    }

    pub(crate) fn to_linear(self, stride: usize) -> usize
    {
        (self.y * stride as isize + self.x) as _
    }

    pub(crate) fn manhattan_distance(&self, other: &Position) -> usize
    {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    // pub(crate) fn same_axis_distance(&self, other: &Position) -> Option<usize>
    // {
    //     if self.y == other.y {
    //         Some(self.x.abs_diff(other.x))
    //     } else if self.x == other.x {
    //         Some(self.y.abs_diff(other.y))
    //     } else {
    //         None
    //     }
    // }
}
