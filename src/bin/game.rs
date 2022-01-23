//! This file contains an experimental game client. It needs to be hooked up to
//! everything still.

use garden::game::game_state::GameState;
use rltk::RltkBuilder;

fn main() -> rltk::BError {
    // Build the terminal.
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let game_state = GameState::new();

    rltk::main_loop(context, game_state)
}
