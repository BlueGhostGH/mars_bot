use ::std::collections;

use crate::{
    game::{self, direction, player, tile},
    io::{input, output::moves},
};

pub(super) use crate::game::Dimensions;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Map
{
    pub(super) dimensions: Dimensions,
    pub(super) entries: Box<[Entry]>,
}

impl Map
{
    pub(super) fn update_with(
        &mut self,
        input::Input {
            dimensions,
            map: input::Map { tiles },
            player:
                game::Player {
                    position,
                    stats: player::stats::Stats { wheel_level, .. },
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
                game::Tile::Fog => *tile,
                _ => incoming_tile,
            };
        }

        self.flood_fill(*position, *wheel_level);
    }

    pub(super) fn flood_fill(&mut self, player: game::Position, wheel_level: u8)
    {
        let mut queue = collections::HashSet::new();

        for x in 0..self.dimensions.width as _ {
            for y in 0..self.dimensions.height as _ {
                let position = game::Position { x, y };
                let _newly_inserted = queue.insert(position);

                // SAFETY: Since both our x and y are
                // within bounds, this entry exists
                let entry @ &mut Entry { tile, .. } =
                    unsafe { self.entry_at_unchecked_mut(position) };

                *entry = Entry {
                    tile,
                    ..Default::default()
                };
            }
        }

        // SAFETY: The player position should never
        // be in an invalid state, if it is then
        // we've got bigger issues than flood-filling
        let source = unsafe { self.entry_at_unchecked_mut(player) };
        source.distance = 0;

        while !queue.is_empty() {
            let nearest_position = queue
                .iter()
                .copied()
                // SAFETY: See the safety note where entries are initialised
                .min_by_key(|&position| unsafe { self.entry_at_unchecked(position) }.distance)
                // SAFETY: The queue is not empty therefore we always have a minimum
                .unwrap();

            let _was_present = queue.remove(&nearest_position);

            // SAFETY: See the safety note where entries are initialised
            let Entry {
                parent_data,
                distance,
                ..
            } = unsafe { self.entry_at_unchecked(nearest_position) }.clone();
            let turn_move_index = parent_data.map_or(0, |parent| {
                (parent.turn_move_index + 1) % (wheel_level as usize)
            });

            for Neighbour {
                direction,
                position,
            } in self
                .neighbours(nearest_position)
                .into_iter()
                .filter(|Neighbour { position, .. }| queue.contains(position))
            {
                if !position.is_within_bounds(self.dimensions) {
                    continue;
                }
                // SAFETY: Since the position is within bounds
                // this entry always exists
                let entry = unsafe { self.entry_at_unchecked_mut(position) };

                let first_move = parent_data.is_none();

                if let Some((weight, requires_mining)) = game::ViableTile::try_from(entry.tile)
                    .ok()
                    .map(|viable_tile| (viable_tile.weight(), viable_tile.requires_mining()))
                {
                    let alternative = distance
                        + if requires_mining && first_move {
                            1000
                        } else if requires_mining && turn_move_index == 0 {
                            1
                        } else {
                            weight
                        };

                    if alternative < entry.distance {
                        *entry = Entry {
                            distance: alternative,
                            parent_data: Some(ParentData {
                                direction_from_parent: direction,
                                parent_location: nearest_position,
                                requires_mining,

                                turn_move_index: if requires_mining { 0 } else { turn_move_index },
                            }),

                            ..*entry
                        };
                    }
                }
            }
        }
    }

    pub(super) fn update_acid(&mut self, level: usize)
    {
        let (level, width, height) = (
            level as _,
            self.dimensions.width as _,
            self.dimensions.height as _,
        );

        let top = (0..width).flat_map(|x| (0..level).map(move |y| (x, y)));
        let bottom = (0..width).flat_map(|x| (height - level..height).map(move |y| (x, y)));
        let left_middle = (0..level).flat_map(|x| (level..height - level).map(move |y| (x, y)));
        let right_middle =
            (width - level..width).flat_map(|x| (level..height - level).map(move |y| (x, y)));

        top.chain(bottom)
            .chain(left_middle)
            .chain(right_middle)
            .map(|(x, y)| game::Position { x, y })
            .for_each(|position| {
                let entry = unsafe { self.entry_at_unchecked_mut(position) };

                entry.tile = game::Tile::Acid;
            });
    }

    pub(crate) fn find_path(
        &self,
        from: game::Position,
        to: game::Position,
        wheel_level: u8,
    ) -> Option<Path>
    {
        let mut location = to;
        let mut moves = collections::VecDeque::new();
        let mut mine_direction = None;

        while location != from {
            let entry = self.entry_at(location)?;

            let ParentData {
                direction_from_parent,
                parent_location,
                requires_mining,

                turn_move_index,
            } = entry.parent_data?;

            if let Some(ParentData {
                direction_from_parent,
                requires_mining,

                turn_move_index,
                ..
            }) = moves.front().copied()
            {
                if turn_move_index == 0 {
                    moves.clear();

                    mine_direction = if requires_mining {
                        Some(direction_from_parent)
                    } else {
                        None
                    };
                }
            }

            moves.push_front(ParentData {
                direction_from_parent,
                parent_location: location,
                requires_mining,

                turn_move_index,
            });

            location = parent_location;

            if moves.len() > wheel_level as _ {
                let _last = moves.pop_back();
            }
        }

        let end_position = moves.front().map_or(
            to,
            |&ParentData {
                 parent_location, ..
             }| parent_location,
        );

        let moves = moves::Moves {
            mvs: [moves.pop_front(), moves.pop_front(), moves.pop_front()].map(|mv| {
                if let Some(ParentData {
                    direction_from_parent,
                    ..
                }) = mv
                {
                    Some(direction_from_parent)
                } else {
                    None
                }
            }),
        };

        Some(Path {
            moves,
            end_position,
            mine_direction,
        })
    }

    fn entry_at(&self, position: game::Position) -> Option<&Entry>
    {
        self.entries.get(position.to_linear(self.dimensions.width))
    }

    pub(super) fn tile_at_is<P>(&self, position: game::Position, pattern: P) -> bool
    where
        P: tile::Pattern,
    {
        self.entry_at(position)
            .map(|Entry { tile, .. }| (*tile).try_into())
            .is_some_and(|np_tile| np_tile.is_ok_and(|np_tile| pattern.matches(&np_tile)))
    }

    unsafe fn entry_at_unchecked(&self, position: game::Position) -> &Entry
    {
        self.entries
            .get_unchecked(position.to_linear(self.dimensions.width))
    }

    unsafe fn entry_at_unchecked_mut(&mut self, position: game::Position) -> &mut Entry
    {
        self.entries
            .get_unchecked_mut(position.to_linear(self.dimensions.width))
    }

    pub(super) fn distance_to(&self, position: game::Position) -> Option<usize>
    {
        self.entry_at(position)
            .map(|Entry { distance, .. }| *distance)
    }

    pub(super) fn center(&self) -> game::Position
    {
        game::Position {
            x: (self.dimensions.width / 2) as _,
            y: (self.dimensions.height / 2) as _,
        }
    }

    pub(super) fn nearest_tile(&self, np_tile: game::NonPlayerTile) -> Option<game::Position>
    {
        self.find_tiles(np_tile)
            .min_by_key(|&position| self.distance_to(position))
    }

    fn find_tiles(&self, np_tile: game::NonPlayerTile) -> FindTiles<'_>
    {
        FindTiles {
            entries: &self.entries,
            index: 0,

            np_tile,
            width: self.dimensions.width,
        }
    }

    pub(super) fn neighbours(&self, of: game::Position) -> [Neighbour; 4]
    {
        direction::DIRECTIONS.map(|direction| Neighbour {
            direction,
            position: of + direction,
        })
    }

    pub(super) fn find_neighbour<const N: usize>(
        &self,
        of: game::Position,
        np_tiles: [game::NonPlayerTile; N],
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
    pub(super) tile: game::Tile,
    pub(super) distance: usize,

    parent_data: Option<ParentData>,
}

impl Entry
{
    pub(super) fn init(tile: game::Tile) -> Self
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
            tile: game::Tile::Fog,
            distance: usize::MAX / 2,

            parent_data: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ParentData
{
    direction_from_parent: game::Direction,
    parent_location: game::Position,
    requires_mining: bool,

    turn_move_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Path
{
    pub(crate) moves: moves::Moves,
    pub(crate) end_position: game::Position,
    pub(crate) mine_direction: Option<game::Direction>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct FindTiles<'entries>
{
    entries: &'entries [Entry],
    index: usize,

    np_tile: game::NonPlayerTile,
    width: usize,
}

impl<'entries> Iterator for FindTiles<'entries>
{
    type Item = game::Position;

    fn next(&mut self) -> Option<Self::Item>
    {
        while let Some(Entry { tile, .. }) = self.entries.take_first() {
            if *tile == self.np_tile {
                let position = game::Position::from_linear(self.index, self.width);

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
    pub(super) direction: game::Direction,
    pub(super) position: game::Position,
}
