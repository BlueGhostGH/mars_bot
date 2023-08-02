#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Map
{
    tiles: Box<[tile::Tile]>,
}

pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Map, Error>
where
    In: AsRef<str>,
{
    Ok(Map {
        tiles: input
            .as_ref()
            .bytes()
            .filter_map(|by| match by {
                b' ' => None,
                _ => Some(by),
            })
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

mod tile
{
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub(super) enum Tile
    {
        Air,
        Stone,
        Cobblestone,
        Bedrock,
        Iron,
        Osmium,
        Base,
        Acid,

        Player
        {
            id: u8,
        },

        #[default]
        Fog,
    }

    pub(super) fn try_parse(input: u8) -> ::core::result::Result<Tile, Error>
    {
        let tile = match input {
            b'.' => Tile::Air,
            b'X' => Tile::Stone,
            b'A' => Tile::Cobblestone,
            b'B' => Tile::Bedrock,
            b'C' => Tile::Iron,
            b'D' => Tile::Osmium,
            b'E' => Tile::Base,
            b'F' => Tile::Acid,

            b'0'..=b'9' => Tile::Player {
                // SAFETY: since our id is in the range 0..=9,
                // converting to an u8 will never fail
                id: u8::try_from(input).unwrap() - 48,
            },

            b'?' => Tile::Fog,
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
