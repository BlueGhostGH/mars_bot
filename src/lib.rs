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

mod game;
mod io;

mod map;
mod opponents;

use crate::io::{input, output};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bot
{
    map: map::Map,

    player: game::Player,
    opponents: opponents::Opponents,

    turn: usize,
    upgrade_queue_index: usize,

    cage: caging::Cage,
}

impl Bot
{
    pub fn turn<In>(&mut self, input: In) -> ::core::result::Result<String, Error>
    where
        In: AsRef<str>,
    {
        let ref input @ input::Input {
            dimensions: game::Dimensions { width, .. },
            map: input::Map { ref tiles },
            player:
                game::Player {
                    position,
                    stats,
                    inventory,
                    ..
                },
        } = input::try_parse(input.as_ref())?;

        self.map.update_acid(self.acid_level());
        self.map.update_with(input);
        self.opponents.update_with(tiles, width);
        self.player = game::Player {
            position,
            stats,
            inventory,

            ..self.player
        };

        let (moves, new_position, mine_direction) = match self.try_move() {
            Some(map::Path {
                moves,
                end_position,
                mine_direction,
            }) => (Some(moves), end_position, mine_direction),
            None => (None, self.player.position, None),
        };

        self.player = game::Player {
            position: new_position,
            ..self.player
        };

        let mine_direction = self
            .map
            .find_neighbour(
                self.player.position,
                [game::NonPlayerTile::Osmium, game::NonPlayerTile::Iron],
            )
            .map(|map::Neighbour { direction, .. }| direction)
            .or(mine_direction)
            .or_else(|| {
                self.map
                    .find_neighbour(
                        self.player.position,
                        [game::NonPlayerTile::Cobblestone, game::NonPlayerTile::Stone],
                    )
                    .map(|map::Neighbour { direction, .. }| direction)
            });

        let action = if let Some(direction) = mine_direction {
            Some(game::Action::Mine { direction })
        } else {
            None
        };

        let upgrade = self.try_upgrade();

        self.turn += 1;
        let output = output::Output {
            moves,
            action,
            upgrade,
        };

        Ok(output::show(output))
    }

    fn try_move(&mut self) -> Option<map::Path>
    {
        if self.acid_level() > 0 {
            let center = self.map.center();

            let map::Neighbour {
                direction: center_direction,
                position: entry,
            } = self
                .map
                .neighbours(center)
                .iter()
                .min_by_key(|map::Neighbour { position, .. }| self.map.distance_to(*position))
                .copied()
                // (UN)SAFETY: In most cases, the map
                // won't be just the centre tile
                .unwrap();

            if entry == self.player.position && self.cage.entryway.is_none() {
                self.cage.entryway = Some(caging::Entryway {
                    center_direction,
                    entry,
                });

                let smaller_dimension = self.map.dimensions.width.min(self.map.dimensions.height);
                self.map.update_acid((smaller_dimension) / 2);
            }

            let next = if let Some(caging::Entryway {
                center_direction,
                entry,
            }) = self.cage.entryway
            {
                let third_spot = center + center_direction.opposite();

                if self.cage.step == 4 {
                    // TODO: Implement going after players once box is built

                    None
                } else if self.player.position == entry
                    && self.map.tile_at_is(center, game::NonPlayerTile::Air)
                    && self.map.tile_at_is(
                        entry + center_direction.clockwise(),
                        game::NonPlayerTile::is_obstacle,
                    )
                    && self.map.tile_at_is(
                        entry + center_direction.counter_clockwise(),
                        game::NonPlayerTile::is_obstacle,
                    )
                {
                    self.cage.step = 1;

                    Some(center)
                } else if self.player.position == center
                    && self.map.tile_at_is(third_spot, game::NonPlayerTile::Air)
                    && self.map.tile_at_is(
                        center + center_direction.clockwise(),
                        game::NonPlayerTile::Air,
                    )
                    && self.map.tile_at_is(
                        center + center_direction.counter_clockwise(),
                        game::NonPlayerTile::Air,
                    )
                {
                    if self.cage.step == 1 {
                        self.cage.step = 2;

                        Some(third_spot)
                    } else {
                        self.cage.step = 4;

                        None
                    }
                } else if self.player.position == third_spot
                    && self.map.tile_at_is(
                        third_spot + center_direction.clockwise(),
                        game::NonPlayerTile::is_obstacle,
                    )
                    && self.map.tile_at_is(
                        third_spot + center_direction.counter_clockwise(),
                        game::NonPlayerTile::is_obstacle,
                    )
                {
                    self.cage.step = 3;

                    Some(center)
                } else {
                    if self.cage.step == 3 {
                        self.cage.step = 4;
                    }

                    None
                }
            } else {
                Some(entry)
            };

            return self
                .map
                .find_path(self.player.position, next?, self.player.stats.wheel_level);
        }

        if self.should_rtb() {
            return self.map.find_path(
                self.player.position,
                self.player.base,
                self.player.stats.wheel_level,
            );
        }

        let nearest = self
            .map
            .nearest_tile(game::NonPlayerTile::Osmium)
            .or_else(|| self.map.nearest_tile(game::NonPlayerTile::Iron))
            .or_else(|| self.map.nearest_tile(game::NonPlayerTile::Fog));

        self.map.find_path(
            self.player.position,
            nearest?,
            self.player.stats.wheel_level,
        )
    }

    fn should_rtb(&self) -> bool
    {
        !self.player.can_upgrade()
            && self
                .target_upgrade()
                .is_some_and(|target| self.player.can_afford(target))
    }

    fn try_upgrade(&mut self) -> Option<game::Upgrade>
    {
        if self.can_upgrade() {
            match self.target_upgrade() {
                Some(upgrade @ game::Upgrade::Heal) => Some(upgrade),
                Some(upgrade) => {
                    self.upgrade_queue_index += 1;

                    Some(upgrade)
                }
                None => None,
            }
        } else {
            None
        }
    }

    fn can_upgrade(&self) -> bool
    {
        self.player.can_upgrade()
            && self
                .target_upgrade()
                .is_some_and(|target| self.player.can_afford(target))
    }

    fn target_upgrade(&self) -> Option<game::Upgrade>
    {
        if self.player.stats.hit_points <= 3 {
            Some(game::Upgrade::Heal)
        } else {
            constants::upgrade::QUEUE
                .get(self.upgrade_queue_index)
                .copied()
        }
    }

    fn acid_level(&self) -> usize
    {
        if self.turn >= constants::acid::START_TURN {
            (self.turn - constants::acid::START_TURN) / constants::acid::TICK_RATE + 1
        } else {
            0
        }
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

mod caging
{
    use crate::game;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) struct Cage
    {
        pub(super) entryway: Option<Entryway>,
        pub(super) step: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Entryway
    {
        pub(super) center_direction: game::Direction,
        pub(super) entry: game::Position,
    }
}

pub mod uninit
{
    use std::collections;

    use crate::{caging, game, io::input, map, opponents};

    pub fn try_init<In>(input: In) -> ::core::result::Result<(crate::Bot, String), crate::Error>
    where
        In: AsRef<str>,
    {
        let ref parsed_input @ input::Input {
            dimensions,
            map: input::Map { ref tiles },
            player:
                game::Player {
                    position,
                    stats,
                    inventory,
                    ..
                },
        } = input::try_parse(input.as_ref())?;

        let entries = tiles.iter().copied().map(map::Entry::init).collect();
        let mut map = map::Map {
            dimensions,
            entries,
        };
        map.update_with(parsed_input);

        let player = game::Player {
            position,
            stats,
            inventory,

            base: position,
        };

        let mut opponents = opponents::Opponents {
            opponents: collections::HashMap::new(),
        };
        opponents.update_with(tiles, dimensions.width);

        let cage = caging::Cage {
            entryway: None,
            step: 0,
        };

        let mut bot = crate::Bot {
            map,
            player,
            opponents,

            turn: 0,
            upgrade_queue_index: 0,

            cage,
        };

        let first_turn = bot.turn(input.as_ref())?;

        Ok((bot, first_turn))
    }
}
