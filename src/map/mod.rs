use crate::{io::input, position};

pub(super) use crate::io::input::dimensions::Dimensions;

mod flood_fill;
pub(super) mod path_finding;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Map
{
    pub(super) dimensions: Dimensions,
    pub(super) entries: Box<[Entry]>,

    pub(super) player: Player,
}

impl Map
{
    pub(super) fn update_with(
        &mut self,
        input::Input {
            dimensions,
            map: input::map::Map { tiles },
            player:
                input::player::Player {
                    position,
                    stats:
                        input::player::stats::Stats {
                            whl_level: wheel_level,
                            ..
                        },
                    ..
                },
        }: &input::Input,
    )
    {
        // TODO: Proper input validation
        assert_eq!(
            &self.dimensions, dimensions,
            "new dimensions don't coincide with the current dimensions"
        );
        assert_eq!(
            self.entries.len(),
            tiles.len(),
            "new map length doesn't coincide with the current map length"
        );

        for (index, Entry { ref mut tile, .. }) in self.entries.iter_mut().enumerate() {
            // SAFETY: It has already been checked that
            // the incoming map's lengh matches
            // the current map's length
            let incoming_tile = *(unsafe { tiles.get_unchecked(index) });

            *tile = match incoming_tile {
                tile::Tile::Fog => *tile,
                _ => incoming_tile,
            };
        }

        self.player = Player {
            position: *position,
            wheel_level: *wheel_level,
        };

        self.flood_fill();
    }

    fn entry_at(&self, position: position::Position) -> Option<&Entry>
    {
        self.entries.get(position.to_linear(self.dimensions.width))
    }

    unsafe fn entry_at_unchecked(&self, position: position::Position) -> &Entry
    {
        self.entries
            .get_unchecked(position.to_linear(self.dimensions.width))
    }

    unsafe fn entry_at_unchecked_mut(&mut self, position: position::Position) -> &mut Entry
    {
        self.entries
            .get_unchecked_mut(position.to_linear(self.dimensions.width))
    }

    fn distance_to(&self, position: position::Position) -> Option<usize>
    {
        self.entry_at(position)
            .map(|Entry { distance, .. }| *distance)
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

    fn neighbours(&self, of: position::Position) -> [Neighbour; 4]
    {
        direction::DIRECTIONS.map(|direction| Neighbour {
            direction,
            position: of + direction,
        })
    }

    pub(super) fn find_neighbour<const N: usize>(
        &self,
        of: position::Position,
        np_tiles: [tile::NonPlayerTile; N],
    ) -> Option<Neighbour>
    {
        np_tiles.iter().find_map(|target| {
            self.neighbours(of)
                .into_iter()
                .find(|&Neighbour { position, .. }| {
                    self.entry_at(position)
                        .is_some_and(|Entry { tile, .. }| tile == target)
                })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Entry
{
    pub(super) tile: tile::Tile,
    pub(super) distance: usize,

    parent_data: Option<ParentData>,
}

impl Entry
{
    pub(super) fn init(tile: tile::Tile) -> Self
    {
        Entry {
            tile,
            ..Default::default()
        }
    }
}

impl Default for Entry
{
    fn default() -> Self
    {
        Entry {
            tile: tile::Tile::Fog,
            distance: usize::MAX / 2,

            parent_data: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ParentData
{
    direction_from_parent: direction::Direction,
    parent_location: position::Position,
    requires_mining: bool,

    turn_move_index: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(super) struct Player
{
    pub(super) position: position::Position,

    pub(super) wheel_level: u8,
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
        while let Some(Entry { tile, .. }) = self.entries.take_first() {
            if *tile == self.np_tile {
                let position = position::Position::from_linear(self.index, self.width);

                self.index += 1;

                return Some(position);
            } else {
                self.index += 1;

                continue;
            };
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Neighbour
{
    pub(super) direction: direction::Direction,
    position: position::Position,
}

mod direction
{
    pub(super) use crate::{io::output::direction::Direction, position};

    pub(super) const DIRECTIONS: [Direction; 4] = [
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
    ];

    impl ::core::ops::Add<Direction> for position::Position
    {
        type Output = position::Position;

        fn add(self, direction: Direction) -> Self::Output
        {
            let position::Position { x, y } = self;

            match direction {
                Direction::Right => position::Position { x: x + 1, y },
                Direction::Up => position::Position { x, y: y - 1 },
                Direction::Left => position::Position { x: x - 1, y },
                Direction::Down => position::Position { x, y: y + 1 },
            }
        }
    }
}

pub(crate) mod tile
{

    pub(crate) use crate::io::input::map::tile::Tile;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) enum NonPlayerTile
    {
        #[allow(dead_code)]
        Air,
        #[allow(dead_code)]
        Base,

        Cobblestone,
        Stone,
        Iron,
        Osmium,

        #[allow(dead_code)]
        Bedrock,
        #[allow(dead_code)]
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

    impl PartialEq<NonPlayerTile> for Tile
    {
        fn eq(&self, np_tile: &NonPlayerTile) -> bool
        {
            use NonPlayerTile as NPTile;

            matches!(
                (self, np_tile),
                (Tile::Air, NPTile::Air)
                    | (Tile::Base, NPTile::Base)
                    | (Tile::Cobblestone, NPTile::Cobblestone)
                    | (Tile::Stone, NPTile::Stone)
                    | (Tile::Iron, NPTile::Iron)
                    | (Tile::Osmium, NPTile::Osmium)
                    | (Tile::Bedrock, NPTile::Bedrock)
                    | (Tile::Acid, NPTile::Acid)
                    | (Tile::Fog, NPTile::Fog)
            )
        }
    }
}
