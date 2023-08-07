use std::collections;

use crate::{map::tile, position};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Opponents
{
    pub(super) opponents: collections::HashMap<Id, Opponent>,
}

pub(super) type Id = u8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Opponent
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

impl Opponents
{
    pub(super) fn update_with(&mut self, tiles: &[tile::Tile], width: usize)
    {
        self.outdate_opponents();

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
                let _opponent = self
                    .update_opponent_position(id, position::Position::from_linear(index, width))
                    .or_insert(Opponent::init_with_position(
                        id,
                        position::Position::from_linear(index, width),
                    ));
            })
    }

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
}
