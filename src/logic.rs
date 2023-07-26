use mars_bot::{Dimensions, Map, GameInput};

#[derive(Debug)]
pub struct GameState {
    dimensions: Dimensions,
    map: Map,
}

impl GameState {
   pub fn feed_input(&mut self, input: GameInput)  {
        self.dimensions = input.dimensions;
        self.map = input.map;
    }
}

pub fn update_state(game_input: GameInput, state: &mut GameState)

pub fn magic(game_state: &mut GameState ) -> GameOutput
{
    let mut rng = rand::thread_rng();
    let direction = match rng.gen_range(1..=4) {
        1 => Direction::Right,
        2 => Direction::Up,
        3 => Direction::Left,
        4 => Direction::Down,
        _ => unreachable!(),
    };

    GameOutput {
        moves: Some(Moves::One { first: direction }),
        action: None,
        upgrade: None,
    }
}
