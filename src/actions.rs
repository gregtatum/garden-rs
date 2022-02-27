use std::{borrow::Cow, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    block_chain::SerializedBytes,
    game::{garden::DrawableGarden, input_device::InputDevice, primitives::Position},
    garden::GardenPlot,
    selectors,
    utils::get_timestamp,
    State, Store,
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
    MovePlayer(
        (
            Position, // position
            Position, // move intent
        ),
    ),
}

impl SerializedBytes for ChainAction {
    fn serialized_bytes(&self) -> Cow<[u8]> {
        Cow::from(bincode::serialize(self).expect("Unable to serialize ChainAction."))
    }
}

pub fn create_garden_plot(name: String) -> Action {
    ChainAction::CreatePlot(GardenPlot::new(name)).into()
}

pub fn tick_game() -> Action {
    GameAction::TickGame(get_timestamp()).into()
}

pub fn maybe_move_player(store: &mut Store, input_device: &InputDevice) {
    let position = selectors::get_player_position(store.state());
    if position.is_none() {
        return;
    }
    let position = position.unwrap();

    let mut next_position = position + input_device.move_intent;

    for garden in &(*selectors::get_drawable_gardens(store.state())) {
        if garden.bbox.intersects_point(position) {
            if garden.bbox.left() == next_position.x
                || garden.bbox.right() == next_position.x
                || garden.bbox.top() == next_position.y
                || garden.bbox.bottom() == next_position.y
            {
                next_position = position;
                break;
            }
        }
    }

    if next_position != position {
        store.dispatch(
            ChainAction::MovePlayer((next_position, input_device.move_intent)).into(),
        );
    }
}
