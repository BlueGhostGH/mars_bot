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
    input::{
        self,
        player::{inventory, stats},
    },
    output::{self, action, upgrade},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bot
{
    map: map::Map,

    player: Player,
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
            dimensions: input::dimensions::Dimensions { width, .. },
            map: input::map::Map { ref tiles },
            player:
                input::player::Player {
                    position,
                    stats,
                    inventory,
                },
        } = input::try_parse(input.as_ref())?;

        self.map.update_acid(self.acid_level());
        self.map.update_with(input);
        self.opponents.update_with(tiles, width);
        self.player = Player {
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

        self.player = Player {
            position: new_position,
            ..self.player
        };

        let mine_direction = self
            .map
            .find_neighbour(
                self.player.position,
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
                        self.player.position,
                        [
                            map::tile::NonPlayerTile::Cobblestone,
                            map::tile::NonPlayerTile::Stone,
                        ],
                    )
                    .map(|map::Neighbour { direction, .. }| direction)
            });

        let action = if let Some(direction) = mine_direction {
            Some(action::Action::Mine { direction })
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
                    && self.map.tile_at_is(center, map::tile::NonPlayerTile::Air)
                    && self.map.tile_at_is(
                        entry + center_direction.clockwise(),
                        map::tile::NonPlayerTile::is_obstacle,
                    )
                    && self.map.tile_at_is(
                        entry + center_direction.counter_clockwise(),
                        map::tile::NonPlayerTile::is_obstacle,
                    )
                {
                    self.cage.step = 1;

                    Some(center)
                } else if self.player.position == center
                    && self
                        .map
                        .tile_at_is(third_spot, map::tile::NonPlayerTile::Air)
                    && self.map.tile_at_is(
                        center + center_direction.clockwise(),
                        map::tile::NonPlayerTile::Air,
                    )
                    && self.map.tile_at_is(
                        center + center_direction.counter_clockwise(),
                        map::tile::NonPlayerTile::Air,
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
                        map::tile::NonPlayerTile::is_obstacle,
                    )
                    && self.map.tile_at_is(
                        third_spot + center_direction.counter_clockwise(),
                        map::tile::NonPlayerTile::is_obstacle,
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
                .find_path(self.player.position, next?, self.player.stats.whl_level);
        }

        if self.should_rtb() {
            return self.map.find_path(
                self.player.position,
                self.player.base,
                self.player.stats.whl_level,
            );
        }

        let nearest = self
            .map
            .nearest_tile(map::tile::NonPlayerTile::Osmium)
            .or_else(|| self.map.nearest_tile(map::tile::NonPlayerTile::Iron))
            .or_else(|| self.map.nearest_tile(map::tile::NonPlayerTile::Fog));

        self.map
            .find_path(self.player.position, nearest?, self.player.stats.whl_level)
    }

    fn should_rtb(&self) -> bool
    {
        !self.player.can_upgrade()
            && self
                .target_upgrade()
                .is_some_and(|target| self.player.can_afford(target))
    }

    fn try_upgrade(&mut self) -> Option<upgrade::Upgrade>
    {
        if self.can_upgrade() {
            match self.target_upgrade() {
                Some(upgrade @ upgrade::Upgrade::Heal) => Some(upgrade),
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

    fn target_upgrade(&self) -> Option<output::upgrade::Upgrade>
    {
        if self.player.stats.hit_points <= 3 {
            Some(upgrade::Upgrade::Heal)
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Player
{
    position: position::Position,
    stats: stats::Stats,
    inventory: inventory::Inventory,

    base: position::Position,
}

impl Player
{
    fn can_upgrade(&self) -> bool
    {
        self.position == self.base || self.stats.has_battery
    }

    fn can_afford(&self, upgrade: upgrade::Upgrade) -> bool
    {
        use upgrade::Upgrade as U;

        let threshold = match upgrade {
            U::Sight => constants::upgrade::SIGHT_THRESHOLDS
                .get(self.stats.cmr_level as usize)
                .copied(),
            U::Attack => constants::upgrade::ATTACK_THRESHOLDS
                .get(self.stats.gun_level as usize)
                .copied(),
            U::Drill => constants::upgrade::DRILL_THRESHOLDS
                .get(self.stats.drl_level as usize)
                .copied(),
            U::Movement => constants::upgrade::MOVEMENT_THRESHOLDS
                .get(self.stats.whl_level as usize)
                .copied(),

            U::Radar => Some(constants::upgrade::RADAR_THRESHOLD),
            U::Battery => Some(constants::upgrade::BATTERY_THRESHOLD),

            U::Heal => Some(constants::upgrade::HEAL_THRESHOLD),
        };

        threshold.is_some_and(|upgrade::Cost { iron, osmium }| {
            self.inventory.iron >= iron && self.inventory.osmium >= osmium
        })
    }
}

mod caging
{
    use crate::{io::output, position};

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) struct Cage
    {
        pub(super) entryway: Option<Entryway>,
        pub(super) step: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Entryway
    {
        pub(super) center_direction: output::direction::Direction,
        pub(super) entry: position::Position,
    }
}

pub mod uninit
{
    use std::collections;

    use crate::{self as bot, caging, io::input, opponents};

    pub fn try_init<In>(input: In) -> ::core::result::Result<(bot::Bot, String), bot::Error>
    where
        In: AsRef<str>,
    {
        let ref parsed_input @ input::Input {
            dimensions,
            map: input::map::Map { ref tiles },
            player:
                input::player::Player {
                    position,
                    stats,
                    inventory,
                },
        } = input::try_parse(input.as_ref())?;

        let entries = tiles.iter().copied().map(bot::map::Entry::init).collect();
        let mut map = bot::map::Map {
            dimensions,
            entries,
        };
        map.update_with(parsed_input);

        let player = bot::Player {
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

        let mut bot = bot::Bot {
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
