mod ai;
mod cli;
mod constants;
mod coords;
mod core;
mod game;
mod movegen;
mod network;
mod search;
mod strong;

fn main() -> std::io::Result<()> {
    cli::run()
}
