#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Player
{
    position: position::Position,
    stats: stats::Stats,
    inventory: inventory::Inventory,
}

pub(super) fn try_parse<In>(
    [position, stats, inventory]: [In; 3],
) -> ::core::result::Result<Player, Error>
where
    In: AsRef<str>,
{
    let position = position::try_parse(position.as_ref())?;
    let stats = stats::try_parse(stats.as_ref())?;
    let inventory = inventory::try_parse(inventory.as_ref())?;

    Ok(Player {
        position,
        stats,
        inventory,
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
    pub(super) use crate::game::position::Position;

    pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Position, Error>
    where
        In: AsRef<str>,
    {
        let (x, y) = input
            .as_ref()
            .split_once(' ')
            .ok_or(Error::MissingDelimiter)?;

        Ok(Position {
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
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) struct Stats
    {
        hit_points: u8,

        drl_level: u8,
        gun_level: u8,
        whl_level: u8,
        cmr_level: u8,

        has_antenna: bool,
        has_battery: bool,
    }

    pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Stats, Error>
    where
        In: AsRef<str>,
    {
        let mut stats = input.as_ref().split(' ');

        let try_parse_property = |property: Property| {
            stats
                .next()
                .ok_or(Error::MissingProperty { property })
                .and_then(|level| level.parse::<u8>().map_err(Error::from))
        };

        let [hit_points, drl_level, gun_level, whl_level, cmr_level, has_antenna, has_battery] =
            PROPERTIES.map(try_parse_property);

        Ok(Stats {
            hit_points: hit_points?,

            drl_level: drl_level?,
            gun_level: gun_level?,
            whl_level: whl_level?,
            cmr_level: cmr_level?,

            has_antenna: has_antenna? != 0,
            has_battery: has_battery? != 0,
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Property
    {
        HitPoints,

        DrlLevel,
        GunLevel,
        WhlLevel,
        CmrLevel,

        HasAntenna,
        HasBattery,
    }

    pub(crate) const PROPERTIES: [Property; 7] = [
        Property::HitPoints,
        Property::DrlLevel,
        Property::GunLevel,
        Property::WhlLevel,
        Property::CmrLevel,
        Property::HasAntenna,
        Property::HasBattery,
    ];

    impl ::core::fmt::Display for Property
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Property::HitPoints => write!(f, "hit points"),

                Property::DrlLevel => write!(f, "drill level"),
                Property::GunLevel => write!(f, "gun level"),
                Property::WhlLevel => write!(f, "wheel level"),
                Property::CmrLevel => write!(f, "camera level"),

                Property::HasAntenna => write!(f, "has antenna"),
                Property::HasBattery => write!(f, "hit battery"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error
    {
        Missing,
        MissingProperty
        {
            property: Property,
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
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub(super) struct Inventory
    {
        stone: u16,
        iron: u16,
        osmium: u16,
    }

    pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Inventory, Error>
    where
        In: AsRef<str>,
    {
        let mut stats = input.as_ref().split(' ');

        let try_parse_item = |property: Item| {
            stats
                .next()
                .ok_or(Error::MissingItem { item: property })
                .and_then(|level| level.parse::<u16>().map_err(Error::from))
        };

        let [stone, iron, osmium] = ITEMS.map(try_parse_item);

        Ok(Inventory {
            stone: stone?,
            iron: iron?,
            osmium: osmium?,
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Item
    {
        Stone,
        Iron,
        Osmium,
    }

    pub(crate) const ITEMS: [Item; 3] = [Item::Stone, Item::Iron, Item::Osmium];

    impl ::core::fmt::Display for Item
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
        {
            match self {
                Item::Stone => write!(f, "stone"),
                Item::Iron => write!(f, "iron"),
                Item::Osmium => write!(f, "osmium"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Error
    {
        Missing,
        MissingItem
        {
            item: Item,
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
