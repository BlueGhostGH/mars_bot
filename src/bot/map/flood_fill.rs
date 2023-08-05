use std::collections;

use crate::{
    bot::map::{self, opponent, tile},
    game::position,
};

use super::Neighbour;

impl map::Map
{
    pub(in crate::bot) fn flood_fill(&mut self)
    {
        let mut queue = collections::HashSet::new();

        for x in 0..self.dimensions.width as _ {
            for y in 0..self.dimensions.height as _ {
                let position = position::Position { x, y };
                let _newly_inserted = queue.insert(position);

                // SAFETY: Since both our x and y are
                // within bounds, this entry exists
                let entry @ &mut map::Entry { tile, .. } =
                    unsafe { self.entry_at_unchecked_mut(position) };

                *entry = map::Entry {
                    tile,
                    ..Default::default()
                };
            }
        }

        // SAFETY: The player position should never
        // be in an invalid state, if it is then
        // we've got bigger issues than flood-filling
        let source = unsafe { self.entry_at_unchecked_mut(self.player.position) };
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
            let map::Entry {
                parent_data,
                distance,
                ..
            } = unsafe { self.entry_at_unchecked(nearest_position) }.clone();
            let turn_move_index = parent_data.map_or(0, |parent| {
                (parent.turn_move_index + 1) % (self.player.wheel_level as usize)
            });

            for map::Neighbour {
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

                if let Some((weight, requires_mining)) = ViableTile::try_from(entry.tile)
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
                        *entry = map::Entry {
                            distance: alternative,
                            parent_data: Some(map::ParentData {
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
enum ViableTile
{
    Air,
    Base,

    Cobblestone,
    Stone,
    Iron,
    Osmium,

    Player,

    #[default]
    Fog,
}

impl ViableTile
{
    fn requires_mining(&self) -> bool
    {
        match self {
            ViableTile::Stone | ViableTile::Cobblestone | ViableTile::Iron | ViableTile::Osmium => {
                true
            }
            ViableTile::Fog | ViableTile::Air | ViableTile::Base | ViableTile::Player => false,
        }
    }

    fn weight(&self) -> usize
    {
        match self {
            ViableTile::Air | ViableTile::Base => 5,

            ViableTile::Osmium => 2,
            ViableTile::Iron => 4,
            ViableTile::Stone | ViableTile::Cobblestone => 8,

            ViableTile::Player => 30,

            ViableTile::Fog => 6,
        }
    }
}

impl TryFrom<tile::Tile> for ViableTile
{
    type Error = Option<::core::convert::Infallible>;

    fn try_from(tile: tile::Tile) -> Result<Self, Self::Error>
    {
        match tile {
            tile::Tile::Air => Ok(ViableTile::Air),
            tile::Tile::Base => Ok(ViableTile::Base),

            tile::Tile::Cobblestone => Ok(ViableTile::Cobblestone),
            tile::Tile::Stone => Ok(ViableTile::Stone),
            tile::Tile::Iron => Ok(ViableTile::Iron),
            tile::Tile::Osmium => Ok(ViableTile::Osmium),

            tile::Tile::Player { .. } => Ok(ViableTile::Player),

            tile::Tile::Fog => Ok(ViableTile::Fog),

            _ => Err(None),
        }
    }
}
