#![feature(
    error_in_core,
    inline_const,
    iter_intersperse,
    maybe_uninit_array_assume_init
)]
#![deny(unsafe_code)]
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

mod array;
pub mod game;

use rand::Rng;

#[derive(Debug)]
enum Error {}

pub fn magic(input: game::input::Input) -> game::output::GameOutput
{
    let mut rng = rand::thread_rng();
    let direction = match rng.gen_range(1..=4) {
        1 => game::output::Direction::Right,
        2 => game::output::Direction::Up,
        3 => game::output::Direction::Left,
        4 => game::output::Direction::Down,
        _ => unreachable!(),
    };

    game::output::GameOutput {
        moves: Some(game::output::Moves::One { first: direction }),
        action: None,
        upgrade: None,
    }
}
