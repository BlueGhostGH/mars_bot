use rand::Rng;

use crate::game::{
    input::{self, player},
    output, position,
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

        // TODO: magic now goes here
        let mut rng = rand::thread_rng();
        let direction = match rng.gen_range(1..=4) {
            1 => output::direction::Direction::Right,
            2 => output::direction::Direction::Up,
            3 => output::direction::Direction::Left,
            4 => output::direction::Direction::Down,
            _ => unreachable!(),
        };

        let output = output::Output {
            moves: Some(output::moves::Moves {
                mvs: [Some(direction), None, None],
            }),
            action: None,
            upgrade: None,
        };

        Ok(output::show(output))
    }

    fn moves(&mut self) -> Option<map::path_finding::Path>
    {
        let closest = self
            .map
            .nearest_tile(map::tile::NonPlayerTile::Osmium)
            .or_else(|| self.map.nearest_tile(map::tile::NonPlayerTile::Iron));

        if let Some(closest) = closest {
            self.move_towards(closest)
        } else {
            self.move_towards(
                self.map
                    .nearest_tile(map::tile::NonPlayerTile::Fog)
                    .unwrap(), // TODO: We might just know the whole map
            )
        }
    }

    #[inline]
    fn move_towards(&self, position: position::Position) -> Option<map::path_finding::Path>
    {
        self.map.find_path(position)
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
