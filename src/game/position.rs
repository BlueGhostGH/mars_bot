#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Position
{
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Position
{
    pub(crate) fn from_linear(i: usize, width: usize) -> Self
    {
        Position {
            x: i % width,
            y: i / width,
        }
    }

    pub(crate) fn to_linear(self, width: usize) -> usize
    {
        self.y * width + self.x
    }
}
