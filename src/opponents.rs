use std::collections;

use crate::game;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Opponents
{
    pub(super) opponents: collections::HashMap<Id, Opponent>,
}

pub(super) type Id = u8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Opponent
{
    pub(super) id: Id,
    pub(super) position: game::Position,
    pub(super) stats: Stats,

    pub(super) up_to_date: bool,
}

impl Opponent
{
    fn init_with_position(id: Id, position: game::Position) -> Self
    {
        Opponent {
            id,
            position,

            up_to_date: true,
            ..Default::default()
        }
    }

    fn outdate(&mut self)
    {
        self.up_to_date = false;
    }

    fn update(&mut self, position: game::Position)
    {
        let wheel_level = if self.up_to_date {
            self.position.manhattan_distance(&position)
        } else {
            1
        } as _;

        *self = Opponent {
            id: self.id,
            position,
            stats: Stats {
                rifle_level: self.stats.rifle_level,
                wheel_level,
            },

            up_to_date: true,
        };
    }
}

impl Opponents
{
    pub(super) fn update_with(&mut self, tiles: &[game::Tile], stride: usize)
    {
        self.outdate_opponents();

        tiles
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, tile)| {
                if let game::Tile::Player { id } = tile {
                    Some((game::Position::from_linear(index, stride), id))
                } else {
                    None
                }
            })
            .for_each(|(position, id)| {
                let _opponent = self
                    .opponents
                    .entry(id)
                    .and_modify(|opponent| opponent.update(position))
                    .or_insert(Opponent::init_with_position(id, position));
            });
    }

    pub(super) fn outdate_opponents(&mut self)
    {
        self.opponents.values_mut().for_each(Opponent::outdate);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Stats
{
    pub(super) rifle_level: u8,
    pub(super) wheel_level: u8,
}

impl Default for Stats
{
    fn default() -> Self
    {
        Stats {
            rifle_level: 1,
            wheel_level: 1,
        }
    }
}
