//! This file contains an experimental game client. It needs to be hooked up to
//! everything still.

use std::path::PathBuf;

use garden::{chain_store::FsChainStore, game::game_state::GameState};
use rltk::RltkBuilder;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "game", about = "A terminal game client for The Land")]
struct CliOptions {
    /// The directory the garden files are persisted to.
    #[structopt(parse(from_os_str), default_value = "./.garden")]
    save_path: PathBuf,
}

fn main() -> rltk::BError {
    // Build the terminal.
    let context = RltkBuilder::simple80x50().with_title("Garden").build()?;

    let cli_options = CliOptions::from_args();
    let chain_store = Box::new(
        FsChainStore::try_new(cli_options.save_path)
            .expect("Unable to create the chain store."),
    );
    let game_state = GameState::new(chain_store);

    rltk::main_loop(context, game_state)
}
