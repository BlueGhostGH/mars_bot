#![feature(fs_try_exists)]

use mars_bot as bot;

fn main()
{
    match try_main() {
        Ok(_) => {}
        Err(err) => panic!("{err:?}"),
    }
}

fn try_main() -> ::std::result::Result<(), Error>
{
    let id = parse_id(::std::io::stdin() /*, ::std::io::stdout()*/)?;
    let mut round = 0usize;

    let directory_path = ::std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("."));

    let mut bot = None::<bot::Bot>;

    loop {
        let read_path = format!("{directory_path}/game/s{id}_{round}.txt");

        let exists = ::std::fs::try_exists(&read_path).unwrap_or(false);

        if exists {
            ::std::thread::sleep(::std::time::Duration::from_millis(10));
        } else {
            continue;
        }

        let input = ::std::fs::read_to_string(&read_path)?;
        let write_path = format!("{directory_path}/game/c{id}_{round}.txt");

        let next_turn = match bot {
            Some(ref mut bot) => bot.turn(&input)?,
            None => {
                let (init_bot, next_turn) = bot::uninit::try_init(input)?;

                let _ = bot.insert(init_bot);

                next_turn
            }
        };

        ::std::fs::write(write_path, next_turn)?;

        round += 1;
    }
}

fn parse_id(stdin: ::std::io::Stdin /*, stdout: ::std::io::Stdout*/) -> Result<usize, Error>
{
    // Doesn't work on Windows
    /*
       let mut out = stdout.lock();
       out.write_all(b"Enter ID:")?;
    */
    // This does
    println!("Enter ID:");

    let mut id = String::new();
    while id.is_empty() {
        stdin.read_line(&mut id)?;
    }

    Ok(id.trim().parse()?)
}

#[derive(Debug)]
enum Error
{
    Io
    {
        io_err: ::std::io::Error
    },
    ParseInt
    {
        parse_int_err: ::std::num::ParseIntError,
    },

    Bot
    {
        bot_err: bot::Error
    },
}

impl ::std::fmt::Display for Error
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result
    {
        match self {
            Error::Io { io_err } => write!(f, "{io_err}"),
            Error::ParseInt { parse_int_err } => write!(f, "{parse_int_err}"),
            Error::Bot { bot_err } => write!(f, "{bot_err}"),
        }
    }
}

impl ::std::error::Error for Error
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self {
            Error::Io { io_err } => Some(io_err),
            Error::ParseInt { parse_int_err } => Some(parse_int_err),
            Error::Bot { bot_err } => Some(bot_err),
        }
    }
}

impl From<::std::io::Error> for Error
{
    fn from(io_err: ::std::io::Error) -> Self
    {
        Error::Io { io_err }
    }
}

impl From<::std::num::ParseIntError> for Error
{
    fn from(parse_int_err: ::std::num::ParseIntError) -> Self
    {
        Error::ParseInt { parse_int_err }
    }
}

impl From<bot::Error> for Error
{
    fn from(bot_err: bot::Error) -> Self
    {
        Error::Bot { bot_err }
    }
}
