use crate::game::{input, output};

use rand::Rng;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Bot;

impl Bot
{
    pub fn turn(input: String) -> ::core::result::Result<String, Error>
    {
        let _input = input::try_parse(input)?;

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
