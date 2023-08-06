use std::collections;

use crate::game::{input, position};

pub(super) use crate::game::input::dimensions::Dimensions;

mod flood_fill;
pub(super) mod path_finding;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Map
{
    pub(super) dimensions: Dimensions,
    pub(super) entries: Box<[Entry]>,

    pub(super) player: Player,
    pub(super) opponents: collections::HashMap<opponent::Id, opponent::Opponent>,
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
        let player_entry = unsafe { self.entry_at_unchecked_mut(self.player.position) };
        *player_entry = Entry {
            tile: tile::Tile::Air,
            ..*player_entry
        };

        self.outdate_opponents();
        self.update_opponents_with(tiles);

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
            distance: usize::MAX,
            ..Default::default()
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
        if let [Entry { tile, .. }, ..] = self.entries {
            let position = if *tile == self.np_tile.into() {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Neighbour
{
    direction: direction::Direction,
    position: position::Position,
}

mod direction
{
    pub(super) use crate::game::{output::direction::Direction, position};

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
                Direction::Up => position::Position { x, y: y + 1 },
                Direction::Left => position::Position { x: x - 1, y },
                Direction::Down => position::Position { x, y: y - 1 },
            }
        }
    }
}

pub(crate) mod tile
{

    pub(crate) use crate::game::input::map::tile::Tile;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(crate) enum NonPlayerTile
    {
        Air,
        Base,

        Cobblestone,
        Stone,
        Iron,
        Osmium,

        Bedrock,
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

mod opponent
{
    use std::collections;

    use crate::{
        bot::map::{self, tile},
        game::position,
    };

    pub(super) type Id = u8;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(in crate::bot) struct Opponent
    {
        pub(super) id: u8,
        pub(super) position: position::Position,
        pub(super) stats: Stats,

        pub(super) up_to_date: bool,
    }

    impl Opponent
    {
        fn init_with_position(id: Id, position: position::Position) -> Self
        {
            Opponent {
                id,
                position,
                ..Default::default()
            }
        }

        fn outdate(&mut self)
        {
            self.up_to_date = false;
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) struct Stats
    {
        pub(super) gun_level: u8,
        pub(super) wheel_level: u8,
    }

    impl map::Map
    {
        pub(super) fn outdate_opponents(&mut self)
        {
            self.opponents.values_mut().for_each(Opponent::outdate);
        }

        fn update_opponent_position(
            &mut self,
            id: u8,
            position: position::Position,
        ) -> collections::hash_map::Entry<'_, Id, Opponent>
        {
            self.opponents.entry(id).and_modify(|opponent| {
                *opponent = Opponent {
                    position,

                    up_to_date: true,
                    ..*opponent
                }
            })
        }

        pub(super) fn update_opponents_with(&mut self, tiles: &[tile::Tile])
        {
            tiles
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(index, tile)| {
                    if let tile::Tile::Player { id } = tile {
                        Some((index, id))
                    } else {
                        None
                    }
                })
                .for_each(|(index, id)| {
                    let map::Dimensions { width, .. } = self.dimensions;

                    let _opponent = self
                        .update_opponent_position(id, position::Position::from_linear(index, width))
                        .or_insert(Opponent::init_with_position(
                            id,
                            position::Position::from_linear(index, width),
                        ));
                })
        }
    }
}
