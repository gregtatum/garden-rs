use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{
    block_chain::SerializedBytes, game::primitives::Position, garden::GardenPlot, utils::get_timestamp,
};

pub enum Action {
    Game(GameAction),
    Chain(ChainAction),
}

impl From<GameAction> for Action {
    fn from(other: GameAction) -> Self {
        Self::Game(other)
    }
}

impl From<ChainAction> for Action {
    fn from(other: ChainAction) -> Self {
        Self::Chain(other)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GameAction {
    TickGame(i64),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ChainAction {
    CreatePlot(GardenPlot),
    MovePlayer((Position, i64)),
}

impl SerializedBytes for ChainAction {
    fn serialized_bytes(&self) -> Cow<[u8]> {
        Cow::from(bincode::serialize(self).expect("Unable to serialize ChainAction."))
    }
}

pub fn create_garden_plot(name: String) -> Action {
    ChainAction::CreatePlot(GardenPlot::new(name)).into()
}

pub fn move_player(position: Position, game_tick: i64) -> Action {
    ChainAction::MovePlayer((position, game_tick)).into()
}

pub fn tick_game() -> Action {
    GameAction::TickGame(get_timestamp()).into()
}
