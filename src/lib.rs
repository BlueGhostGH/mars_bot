#![feature(
    array_try_map,
    error_in_core,
    inline_const,
    iter_intersperse,
    maybe_uninit_array_assume_init,
    try_blocks
)]
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

pub mod bot;
mod game;
