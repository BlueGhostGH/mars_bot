use crate::game;

use rand::Rng;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Bot;

impl Bot
{
    pub fn turn(input: String) -> String
    {
        let _input = game::input::try_parse(input);

        // TODO: magic now goes here
        let mut rng = rand::thread_rng();
        let direction = match rng.gen_range(1..=4) {
            1 => game::output::direction::Direction::Right,
            2 => game::output::direction::Direction::Up,
            3 => game::output::direction::Direction::Left,
            4 => game::output::direction::Direction::Down,
            _ => unreachable!(),
        };

        let output = game::output::Output {
            moves: Some(game::output::moves::Moves {
                mvs: [Some(direction), None, None],
            }),
            action: None,
            upgrade: None,
        };

        game::output::show(output)
    }
}
