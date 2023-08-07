#![feature(
    array_try_map,
    error_in_core,
    inline_const,
    iter_intersperse,
    maybe_uninit_array_assume_init,
    slice_take,
    try_blocks
)]
#![warn(
    clippy::all,
    explicit_outlives_requirements,
    let_underscore_drop,
    missing_copy_implementations,
    missing_debug_implementations,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    unreachable_pub,
    unused_results,
    variant_size_differences
)]
#![allow(clippy::new_without_default)]

mod constants;

mod io;
mod map;
mod opponents;

mod position;

use io::{
    input::{self, player},
    output,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bot
{
    map: map::Map,

    player: player::Player,
    opponents: opponents::Opponents,

    turn: usize,
}

impl Bot
{
    pub fn turn<In>(&mut self, input: In) -> ::core::result::Result<String, Error>
    where
        In: AsRef<str>,
    {
        let ref input @ input::Input {
            dimensions: input::dimensions::Dimensions { width, .. },
            map: input::map::Map { ref tiles },
            player,
        } = input::try_parse(input.as_ref())?;

        self.map.update_with(&input);
        self.opponents.update_with(&tiles, width);
        self.player = player;

        let path = try {
            let closest = self
                .map
                .nearest_tile(map::tile::NonPlayerTile::Osmium)
                .or_else(|| self.map.nearest_tile(map::tile::NonPlayerTile::Iron));

            if let Some(closest) = closest {
                self.map.find_path(closest)?
            } else {
                self.map
                    .find_path(self.map.nearest_tile(map::tile::NonPlayerTile::Fog)?)?
            }
        };

        let (moves, new_position, mine_direction) = match path {
            Some(map::path_finding::Path {
                moves,
                end_position,
                mine_direction,
            }) => (Some(moves), end_position, mine_direction),
            None => (None, self.player.position, None),
        };

        let mine_direction = self
            .map
            .find_neighbour(
                new_position,
                [
                    map::tile::NonPlayerTile::Osmium,
                    map::tile::NonPlayerTile::Iron,
                ],
            )
            .map(|map::Neighbour { direction, .. }| direction)
            .or(mine_direction)
            .or_else(|| {
                self.map
                    .find_neighbour(
                        new_position,
                        [
                            map::tile::NonPlayerTile::Cobblestone,
                            map::tile::NonPlayerTile::Stone,
                        ],
                    )
                    .map(|map::Neighbour { direction, .. }| direction)
            });

        let action = if let Some(direction) = mine_direction {
            Some(output::action::Action::Mine { direction })
        } else {
            None
        };

        self.turn += 1;
        let output = output::Output {
            moves,
            action,
            upgrade: None,
        };

        Ok(output::show(output))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error
{
    Input
    {
        input_err: input::Error
    },
}

impl ::core::fmt::Display for Error
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
    {
        match self {
            Error::Input { input_err } => write!(f, "{input_err}"),
        }
    }
}

impl ::core::error::Error for Error
{
    fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
    {
        match self {
            Error::Input { input_err } => Some(input_err),
        }
    }
}

impl From<input::Error> for Error
{
    fn from(input_err: input::Error) -> Self
    {
        Error::Input { input_err }
    }
}

pub mod uninit
{
    use std::collections;

    use crate::{self as bot, io::input, opponents};

    pub fn try_init<In>(input: In) -> ::core::result::Result<(bot::Bot, String), bot::Error>
    where
        In: AsRef<str>,
    {
        let ref parsed_input @ input::Input {
            dimensions: dimensions @ input::dimensions::Dimensions { width, .. },
            map: input::map::Map { ref tiles },
            player:
                player @ input::player::Player {
                    position,
                    stats:
                        input::player::stats::Stats {
                            whl_level: wheel_level,
                            ..
                        },
                    ..
                },
        } = input::try_parse(input.as_ref())?;

        let entries = tiles.iter().copied().map(bot::map::Entry::init).collect();
        let mut map = bot::map::Map {
            dimensions,
            entries,

            player: bot::map::Player {
                position,
                wheel_level,
            },
        };
        map.update_with(&parsed_input);

        let mut opponents = opponents::Opponents {
            opponents: collections::HashMap::new(),
        };
        opponents.update_with(&tiles, dimensions.width);

        let mut bot = bot::Bot {
            map,
            player,
            opponents,

            turn: 0,
        };

        let first_turn = bot.turn(input.as_ref())?;

        Ok((bot, first_turn))
    }
}
