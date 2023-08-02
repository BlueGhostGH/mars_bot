use crate::array::ArrayTransposeResult;

pub(crate) mod dimensions;
pub(crate) mod map;
pub(crate) mod player;

use dimensions::Dimensions;
use map::Map;
use player::Player;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Input
{
    dimensions: Dimensions,
    map: Map,
    player: Player,
}

pub(crate) fn try_parse<In>(input: In) -> ::core::result::Result<Input, Error>
where
    In: AsRef<str>,
{
    let game = input.as_ref().trim().lines().collect::<Vec<_>>();

    let dimensions = dimensions::try_parse(game.get(0).ok_or(dimensions::Error::Missing)?)?;
    let game = game.get(1..).ok_or(map::Error::Missing)?;

    let map = map::try_parse(
        game.get(..dimensions.height)
            .ok_or(map::Error::Incomplete {
                kind: map::IncompleteKind::MissingRow,
            })?
            .into_iter()
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
            .intersperse(Ok(' '.into()))
            .collect::<Result<String, map::Error>>()?,
    )?;
    let game = game.get(dimensions.height..).unwrap_or_default();

    let player = player::try_parse(
        [
            (0, player::position::Error::Missing.into()),
            (1, player::stats::Error::Missing.into()),
            (2, player::inventory::Error::Missing.into()),
        ]
        .map(|(index, err)| game.get(index).copied().ok_or::<player::Error>(err))
        .transpose_result()?,
    )?;

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
