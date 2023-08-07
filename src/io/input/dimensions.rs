#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Dimensions
{
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub(super) fn try_parse<In>(input: In) -> ::core::result::Result<Dimensions, Error>
where
    In: AsRef<str>,
{
    let (width, height) = input
        .as_ref()
        .split_once(' ')
        .ok_or(Error::MissingDelimiter)?;

    Ok(Dimensions {
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
