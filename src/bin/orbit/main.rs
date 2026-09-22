#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod commands;
mod error;
mod runner;

fn main() -> error::ExitCode {
    runner::run()
}
