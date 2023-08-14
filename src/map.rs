use ::std::collections;

use crate::{
    io::{input, output::moves},
    position,
};

pub(super) use crate::io::input::dimensions::Dimensions;

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

        self.flood_fill(*position, *wheel_level);
    }

    pub(super) fn flood_fill(&mut self, player: position::Position, wheel_level: u8)
    {
        let mut queue = collections::HashSet::new();

        for x in 0..self.dimensions.width as _ {
            for y in 0..self.dimensions.height as _ {
                let position = position::Position { x, y };
                let _newly_inserted = queue.insert(position);

                // SAFETY: Since both our x and y are
                // within bounds, this entry exists
                let entry @ &mut Entry { tile, .. } =
                    unsafe { self.entry_at_unchecked_mut(position) };
                entry.tile = tile;
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

                if let Some((weight, requires_mining)) = tile::ViableTile::try_from(entry.tile)
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
            .map(|(x, y)| position::Position { x, y })
            .for_each(|position| {
                let entry = unsafe { self.entry_at_unchecked_mut(position) };

                entry.tile = tile::Tile::Acid;
            });
    }

    pub(crate) fn find_path(
        &self,
        from: position::Position,
        to: position::Position,
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

    fn entry_at(&self, position: position::Position) -> Option<&Entry>
    {
        self.entries.get(position.to_linear(self.dimensions.width))
    }

    pub(super) fn tile_at_is<P>(&self, position: position::Position, pattern: P) -> bool
    where
        P: tile::Pattern,
    {
        self.entry_at(position)
            .map(|Entry { tile, .. }| (*tile).try_into())
            .is_some_and(|np_tile| np_tile.is_ok_and(|np_tile| pattern.matches(&np_tile)))
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

    pub(super) fn distance_to(&self, position: position::Position) -> Option<usize>
    {
        self.entry_at(position)
            .map(|Entry { distance, .. }| *distance)
    }

    pub(super) fn center(&self) -> position::Position
    {
        position::Position {
            x: (self.dimensions.width / 2) as _,
            y: (self.dimensions.height / 2) as _,
        }
    }

    pub(super) fn nearest_tile(&self, np_tile: tile::NonPlayerTile) -> Option<position::Position>
    {
        self.find_tiles(np_tile)
            .min_by_key(|&position| self.distance_to(position))
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

    pub(super) fn neighbours(&self, of: position::Position) -> [Neighbour; 4]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Path
{
    pub(crate) moves: moves::Moves,
    pub(crate) end_position: position::Position,
    pub(crate) mine_direction: Option<direction::Direction>,
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
    pub(super) position: position::Position,
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

    use crate::constants::tile_weights;

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

    impl NonPlayerTile
    {
        pub(crate) fn is_obstacle(&self) -> bool
        {
            match self {
                NonPlayerTile::Acid
                | NonPlayerTile::Bedrock
                | NonPlayerTile::Cobblestone
                | NonPlayerTile::Stone => true,
                _ => false,
            }
        }
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

    impl TryFrom<Tile> for NonPlayerTile
    {
        type Error = Tile;

        fn try_from(tile: Tile) -> Result<Self, Self::Error>
        {
            match tile {
                Tile::Air => Ok(NonPlayerTile::Air),
                Tile::Stone => Ok(NonPlayerTile::Stone),
                Tile::Cobblestone => Ok(NonPlayerTile::Cobblestone),
                Tile::Bedrock => Ok(NonPlayerTile::Bedrock),
                Tile::Iron => Ok(NonPlayerTile::Iron),
                Tile::Osmium => Ok(NonPlayerTile::Osmium),
                Tile::Base => Ok(NonPlayerTile::Base),
                Tile::Acid => Ok(NonPlayerTile::Acid),

                _ => Err(tile),
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

    pub(crate) trait Pattern
    {
        fn matches(&self, np_tile: &NonPlayerTile) -> bool;
    }

    impl Pattern for NonPlayerTile
    {
        fn matches(&self, np_tile: &NonPlayerTile) -> bool
        {
            self == np_tile
        }
    }

    impl<F> Pattern for F
    where
        F: Fn(&NonPlayerTile) -> bool,
    {
        fn matches(&self, np_tile: &NonPlayerTile) -> bool
        {
            self(np_tile)
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) enum ViableTile
    {
        Air,
        Base,

        Cobblestone,
        Stone,
        Iron,
        Osmium,

        Acid,

        Player,

        #[default]
        Fog,
    }

    impl ViableTile
    {
        pub(super) fn requires_mining(&self) -> bool
        {
            match self {
                ViableTile::Stone
                | ViableTile::Cobblestone
                | ViableTile::Iron
                | ViableTile::Osmium => true,
                ViableTile::Fog
                | ViableTile::Air
                | ViableTile::Base
                | ViableTile::Acid
                | ViableTile::Player => false,
            }
        }

        pub(super) fn weight(&self) -> usize
        {
            match self {
                ViableTile::Air => tile_weights::AIR,
                ViableTile::Base => tile_weights::BASE,

                ViableTile::Osmium => tile_weights::OSMIUM,
                ViableTile::Iron => tile_weights::IRON,
                ViableTile::Stone => tile_weights::STONE,
                ViableTile::Cobblestone => tile_weights::COBBLESTONE,

                ViableTile::Acid => tile_weights::ACID,

                ViableTile::Player => tile_weights::PLAYER,

                ViableTile::Fog => tile_weights::FOG,
            }
        }
    }

    impl TryFrom<Tile> for ViableTile
    {
        type Error = Option<::core::convert::Infallible>;

        fn try_from(tile: Tile) -> Result<Self, Self::Error>
        {
            match tile {
                Tile::Air => Ok(ViableTile::Air),
                Tile::Base => Ok(ViableTile::Base),

                Tile::Cobblestone => Ok(ViableTile::Cobblestone),
                Tile::Stone => Ok(ViableTile::Stone),
                Tile::Iron => Ok(ViableTile::Iron),
                Tile::Osmium => Ok(ViableTile::Osmium),

                Tile::Acid => Ok(ViableTile::Acid),

                Tile::Player { .. } => Ok(ViableTile::Player),

                Tile::Fog => Ok(ViableTile::Fog),

                _ => Err(None),
            }
        }
    }
}
