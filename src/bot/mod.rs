use crate::game::{
    input::{self, player},
    output,
};

mod map;
mod opponents;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bot
{
    map: map::Map,

    player: player::Player,
    opponents: opponents::Opponents,
}

impl Bot
{
    pub fn turn<In>(&self, input: In) -> ::core::result::Result<String, Error>
    where
        In: AsRef<str>,
    {
        let _input = input::try_parse(input.as_ref())?;

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

        let (moves, _new_position, mine_direction) = match path {
            Some(map::path_finding::Path {
                moves,
                end_position,
                mine_direction,
            }) => (Some(moves), end_position, mine_direction),
            None => {
                let mineable_neighbour = self.map.find_neighbour(
                    self.player.position,
                    [
                        map::tile::NonPlayerTile::Osmium,
                        map::tile::NonPlayerTile::Iron,
                        map::tile::NonPlayerTile::Stone,
                        map::tile::NonPlayerTile::Cobblestone,
                    ],
                );

                (
                    None,
                    self.player.position,
                    mineable_neighbour.map(|map::Neighbour { direction, .. }| direction),
                )
            }
        };

        let action = if let Some(direction) = mine_direction {
            Some(output::action::Action::Mine { direction })
        } else {
            None
        };

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

    use crate::{
        bot::{self, opponents},
        game::input,
    };

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
        opponents.outdate_opponents();
        opponents.update_opponents_with(&tiles, dimensions.width);

        let bot = bot::Bot {
            map,
            player,
            opponents,
        };

        let first_turn = bot.turn(input.as_ref())?;

        Ok((bot, first_turn))
    }
}
