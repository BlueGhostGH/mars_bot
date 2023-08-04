use crate::game::position;

pub(super) use crate::game::input::dimensions::Dimensions;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(super) struct Map
{
    pub(super) dimensions: Dimensions,
    pub(super) entries: Box<[Entry]>,
}

impl Map
{
    fn entry_at(&self, position: position::Position) -> Option<Entry>
    {
        self.entries
            .get(position.to_linear(self.dimensions.width))
            .copied()
    }

    fn distance_to(&self, position: position::Position) -> Option<usize>
    {
        self.entry_at(position)
            .map(|Entry { distance, .. }| distance)
    }

    fn find_tiles(&self, np_tile: tile::NonPlayerTile) -> FindTiles<'_>
    {
        FindTiles {
            entries: &self.entries,
            index: 0,

            np_tile,
            width: self.dimensions.width,
        }
    }

    pub(super) fn nearest_tile(&self, np_tile: tile::NonPlayerTile) -> Option<position::Position>
    {
        self.find_tiles(np_tile)
            .min_by_key(|&position| self.distance_to(position))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(super) struct Entry
{
    pub(super) tile: tile::Tile,
    pub(super) distance: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct FindTiles<'entries>
{
    entries: &'entries [Entry],
    index: usize,

    np_tile: tile::NonPlayerTile,
    width: usize,
}

impl<'entries> Iterator for FindTiles<'entries>
{
    type Item = position::Position;

    fn next(&mut self) -> Option<Self::Item>
    {
        if let [Entry { tile, .. }, ..] = *self.entries {
            let position = if tile == self.np_tile.into() {
                Some(position::Position::from_linear(self.index, self.width))
            } else {
                None
            };

            self.entries = &self.entries[1..];
            self.index += 1;

            position
        } else {
            None
        }
    }
}

mod direction
{
    use crate::game::output::direction::Direction;

    impl Direction {}
}

pub(crate) mod tile
{

    pub(crate) use crate::game::input::map::tile::Tile;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) enum NonPlayerTile
    {
        Air,
        Stone,
        Cobblestone,
        Bedrock,
        Iron,
        Osmium,
        Base,
        Acid,

        #[default]
        Fog,
    }

    impl From<NonPlayerTile> for Tile
    {
        fn from(non_player_tile: NonPlayerTile) -> Self
        {
            match non_player_tile {
                NonPlayerTile::Air => Tile::Air,
                NonPlayerTile::Stone => Tile::Stone,
                NonPlayerTile::Cobblestone => Tile::Cobblestone,
                NonPlayerTile::Bedrock => Tile::Bedrock,
                NonPlayerTile::Iron => Tile::Iron,
                NonPlayerTile::Osmium => Tile::Osmium,
                NonPlayerTile::Base => Tile::Base,
                NonPlayerTile::Acid => Tile::Acid,

                NonPlayerTile::Fog => Tile::Fog,
            }
        }
    }
}
