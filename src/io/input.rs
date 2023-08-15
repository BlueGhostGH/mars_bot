use crate::game;

pub(crate) use map::Map;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Input
{
    pub(crate) dimensions: game::Dimensions,
    pub(crate) map: Map,
    pub(crate) player: game::Player,
}

pub(crate) fn try_parse<In>(input: In) -> ::core::result::Result<Input, Error>
where
    In: AsRef<str>,
{
    let game = input.as_ref().trim().lines().collect::<Vec<_>>();

    let dimensions = dimensions::try_parse(game.first().ok_or(dimensions::Error::Missing)?)?;
    let game = game.get(1..).ok_or(map::Error::Missing)?;

    let mut map = map::try_parse(
        game.get(..dimensions.height)
            .ok_or(map::Error::Incomplete {
                kind: map::IncompleteKind::MissingRow,
            })?
            .iter()
            .map(|row| {
                let mut row = row
                    .bytes()
                    .filter(|by| !by.is_ascii_whitespace())
                    .collect::<Vec<_>>();

                if row.len() >= dimensions.width {
                    row.truncate(dimensions.width);

                    // SAFETY: as the bytes in `row: Vec<u8>` are coming from
                    // `row: &str`, we know they are valid UTF-8
                    Ok(String::from_utf8(row).unwrap())
                } else {
                    Err(map::Error::Incomplete {
                        kind: map::IncompleteKind::IncompleteRow,
                    })
                }
            })
            .collect::<Result<String, map::Error>>()?,
    )?;
    let game = game.get(dimensions.height..).unwrap_or_default();

    let player = player::try_parse(
        [
            (0, player::position::Error::Missing.into()),
            (1, player::stats::Error::Missing.into()),
            (2, player::inventory::Error::Missing.into()),
        ]
        .try_map(|(index, err)| game.get(index).copied().ok_or::<player::Error>(err))?,
    )?;

    // NOTE: We treat our player's tile as `tile::Air`
    *(unsafe {
        map.tiles
            .get_unchecked_mut(player.position.to_linear(dimensions.width))
    }) = game::Tile::Air;

    Ok(Input {
        dimensions,
        map,
        player,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error
{
    Dimensions(dimensions::Error),
    Map(map::Error),
    Player(player::Error),
}

impl ::core::fmt::Display for Error
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
    {
        match self {
            Error::Dimensions(dimensions_err) => write!(f, "{dimensions_err}"),
            Error::Map(map_err) => write!(f, "{map_err}"),
            Error::Player(player_err) => write!(f, "{player_err}"),
        }
    }
}

impl ::core::error::Error for Error
{
    fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
    {
        match self {
            Error::Dimensions(dimensions_err) => Some(dimensions_err),
            Error::Map(map_err) => Some(map_err),
            Error::Player(player_err) => Some(player_err),
        }
    }
}

impl From<dimensions::Error> for Error
{
    fn from(dimensions_err: dimensions::Error) -> Self
    {
        Error::Dimensions(dimensions_err)
    }
}

impl From<map::Error> for Error
{
    fn from(map_err: map::Error) -> Self
    {
        Error::Map(map_err)
    }
}

impl From<player::Error> for Error
{
    fn from(player_err: player::Error) -> Self
    {
        Error::Player(player_err)
    }
}

mod dimensions
{
    use crate::game;

    pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<game::Dimensions, Error>
    where
        In: AsRef<str>,
    {
        let (width, height) = input
            .as_ref()
            .split_once(' ')
            .ok_or(Error::MissingDelimiter)?;

        Ok(game::Dimensions {
            width: width.parse()?,
            height: height.parse()?,
        })
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error
    {
        Missing,
        MissingDelimiter,
        ParseInt
        {
            parse_int_err: ::core::num::ParseIntError,
        },
    }

    impl ::core::fmt::Display for Error
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Error::Missing => write!(f, "missing dimensions"),
                Error::MissingDelimiter => write!(f, "missing dimensions delimiter"),
                Error::ParseInt { parse_int_err } => write!(f, "{parse_int_err}"),
            }
        }
    }

    impl ::core::error::Error for Error
    {
        fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
        {
            match self {
                Error::ParseInt { parse_int_err } => Some(parse_int_err),
                Error::Missing | Error::MissingDelimiter => None,
            }
        }
    }

    impl From<::core::num::ParseIntError> for Error
    {
        fn from(parse_int_err: ::core::num::ParseIntError) -> Self
        {
            Error::ParseInt { parse_int_err }
        }
    }
}

mod map
{
    use crate::game;

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub(crate) struct Map
    {
        pub(crate) tiles: Box<[game::Tile]>,
    }

    pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Map, Error>
    where
        In: AsRef<str>,
    {
        Ok(Map {
            tiles: input
                .as_ref()
                .bytes()
                .map(tile::try_parse)
                .collect::<Result<Box<[_]>, tile::Error>>()?,
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum IncompleteKind
    {
        IncompleteRow,
        MissingRow,
    }

    impl ::core::fmt::Display for IncompleteKind
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                IncompleteKind::IncompleteRow => write!(f, "incomplete row"),
                IncompleteKind::MissingRow => write!(f, "missing row"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Error
    {
        Missing,
        Incomplete
        {
            kind: IncompleteKind,
        },
        Tile
        {
            tile_err: tile::Error,
        },
    }

    impl ::core::fmt::Display for Error
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Error::Missing => write!(f, "missing map"),
                Error::Incomplete { kind } => write!(f, "incomplete map, {kind}"),
                Error::Tile { tile_err } => write!(f, "{tile_err}"),
            }
        }
    }

    impl ::core::error::Error for Error
    {
        fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
        {
            match self {
                Error::Tile { tile_err } => Some(tile_err),
                Error::Missing | Error::Incomplete { .. } => None,
            }
        }
    }

    impl From<tile::Error> for Error
    {
        fn from(tile_err: tile::Error) -> Self
        {
            Error::Tile { tile_err }
        }
    }

    pub(crate) mod tile
    {

        use crate::game;

        pub(super) fn try_parse(input: u8) -> ::core::result::Result<game::Tile, Error>
        {
            use game::Tile as T;

            let tile = match input {
                b'.' => T::Air,
                b'X' => T::Stone,
                b'A' => T::Cobblestone,
                b'B' => T::Bedrock,
                b'C' => T::Iron,
                b'D' => T::Osmium,
                b'E' => T::Base,
                b'F' => T::Acid,

                b'0'..=b'9' => T::Player { id: input - 48 },

                b'?' => T::Fog,
                tile => return Err(Error::Unknown { tile }),
            };

            Ok(tile)
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Error
        {
            Unknown
            {
                tile: u8
            },
        }

        impl ::core::fmt::Display for Error
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
            {
                match self {
                    Error::Unknown { tile } => write!(f, "unknown tile {}", *tile),
                }
            }
        }

        impl ::core::error::Error for Error
        {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
            {
                match self {
                    Error::Unknown { .. } => None,
                }
            }
        }
    }
}

mod player
{
    use crate::game::player;

    pub(super) fn try_parse<In>(
        [position, stats, inventory]: [In; 3],
    ) -> ::core::result::Result<player::Player, Error>
    where
        In: AsRef<str>,
    {
        let position = position::try_parse(position.as_ref())?;
        let stats = stats::try_parse(stats.as_ref())?;
        let inventory = inventory::try_parse(inventory.as_ref())?;

        Ok(player::Player {
            position,
            stats,
            inventory,

            base: position,
        })
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error
    {
        Position
        {
            position_err: position::Error
        },
        Stats
        {
            stats_err: stats::Error
        },
        Inventory
        {
            inventory_err: inventory::Error
        },
    }

    impl ::core::fmt::Display for Error
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Error::Position { position_err } => write!(f, "{position_err}"),
                Error::Stats { stats_err } => write!(f, "{stats_err}"),
                Error::Inventory { inventory_err } => write!(f, "{inventory_err}"),
            }
        }
    }

    impl ::core::error::Error for Error
    {
        fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
        {
            match self {
                Error::Position { position_err } => Some(position_err),
                Error::Stats { stats_err } => Some(stats_err),
                Error::Inventory { inventory_err } => Some(inventory_err),
            }
        }
    }

    impl From<position::Error> for Error
    {
        fn from(position_err: position::Error) -> Self
        {
            Error::Position { position_err }
        }
    }

    impl From<stats::Error> for Error
    {
        fn from(stats_err: stats::Error) -> Self
        {
            Error::Stats { stats_err }
        }
    }

    impl From<inventory::Error> for Error
    {
        fn from(inventory_err: inventory::Error) -> Self
        {
            Error::Inventory { inventory_err }
        }
    }

    pub(crate) mod position
    {
        pub(super) use crate::game::position;

        pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<position::Position, Error>
        where
            In: AsRef<str>,
        {
            let (x, y) = input
                .as_ref()
                .split_once(' ')
                .ok_or(Error::MissingDelimiter)?;

            Ok(position::Position {
                x: x.parse()?,
                y: y.parse()?,
            })
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Error
        {
            Missing,
            MissingDelimiter,
            ParseInt
            {
                parse_int_err: ::core::num::ParseIntError,
            },
        }

        impl ::core::fmt::Display for Error
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
            {
                match self {
                    Error::Missing => write!(f, "missing dimensions"),
                    Error::MissingDelimiter => write!(f, "missing dimensions delimiter"),
                    Error::ParseInt { parse_int_err } => write!(f, "{parse_int_err}"),
                }
            }
        }

        impl ::core::error::Error for Error
        {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
            {
                match self {
                    Error::ParseInt { parse_int_err } => Some(parse_int_err),
                    Error::Missing | Error::MissingDelimiter => None,
                }
            }
        }

        impl From<::core::num::ParseIntError> for Error
        {
            fn from(parse_int_err: ::core::num::ParseIntError) -> Self
            {
                Error::ParseInt { parse_int_err }
            }
        }
    }

    pub(crate) mod stats
    {
        use crate::game::player::{self, stats};

        pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<player::Stats, Error>
        where
            In: AsRef<str>,
        {
            let mut stats = input.as_ref().split(' ');

            let try_parse_property = |property: stats::Property| {
                stats
                    .next()
                    .ok_or(Error::MissingProperty { property })
                    .and_then(|level| level.parse::<u8>().map_err(Error::from))
            };

            let [hit_points, drill_level, rifle_level, wheel_level, sight_level, has_antenna, has_battery] =
                stats::PROPERTIES.try_map(try_parse_property)?;

            Ok(player::Stats {
                hit_points,

                drill_level,
                rifle_level,
                wheel_level,
                sight_level,

                has_antenna: has_antenna != 0,
                has_battery: has_battery != 0,
            })
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Error
        {
            Missing,
            MissingProperty
            {
                property: stats::Property,
            },
            ParseInt
            {
                parse_int_err: ::core::num::ParseIntError,
            },
        }

        impl ::core::fmt::Display for Error
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
            {
                match self {
                    Error::Missing => write!(f, "missing dimensions"),
                    Error::MissingProperty { property } => write!(f, "missing property {property}"),
                    Error::ParseInt { parse_int_err } => write!(f, "{parse_int_err}"),
                }
            }
        }

        impl ::core::error::Error for Error
        {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
            {
                match self {
                    Error::ParseInt { parse_int_err } => Some(parse_int_err),
                    Error::Missing | Error::MissingProperty { .. } => None,
                }
            }
        }

        impl From<::core::num::ParseIntError> for Error
        {
            fn from(parse_int_err: ::core::num::ParseIntError) -> Self
            {
                Error::ParseInt { parse_int_err }
            }
        }
    }

    pub(crate) mod inventory
    {
        use crate::game::player::{self, inventory};

        pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<player::Inventory, Error>
        where
            In: AsRef<str>,
        {
            let mut stats = input.as_ref().split(' ');

            let try_parse_item = |property: inventory::Item| {
                stats
                    .next()
                    .ok_or(Error::MissingItem { item: property })
                    .and_then(|level| level.parse::<u16>().map_err(Error::from))
            };

            let [stone, iron, osmium] = inventory::ITEMS.try_map(try_parse_item)?;

            Ok(player::Inventory {
                stone,
                iron,
                osmium,
            })
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Error
        {
            Missing,
            MissingItem
            {
                item: inventory::Item,
            },
            ParseInt
            {
                parse_int_err: ::core::num::ParseIntError,
            },
        }

        impl ::core::fmt::Display for Error
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
            {
                match self {
                    Error::Missing => write!(f, "missing dimensions"),
                    Error::MissingItem { item } => write!(f, "missing property {item}"),
                    Error::ParseInt { parse_int_err } => write!(f, "{parse_int_err}"),
                }
            }
        }

        impl ::core::error::Error for Error
        {
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)>
            {
                match self {
                    Error::ParseInt { parse_int_err } => Some(parse_int_err),
                    Error::Missing | Error::MissingItem { .. } => None,
                }
            }
        }

        impl From<::core::num::ParseIntError> for Error
        {
            fn from(parse_int_err: ::core::num::ParseIntError) -> Self
            {
                Error::ParseInt { parse_int_err }
            }
        }
    }
}
