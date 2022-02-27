use crate::{
    combine_reducers, game::primitives::Position, garden::GardenPlot, reducers, Action,
};
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub struct State {
    my_garden: Option<Rc<GardenPlot>>,
    game_tick: Option<i64>,
    player_position: Option<Position>,
}

impl State {
    pub fn new() -> Self {
        Self {
            my_garden: None,
            game_tick: Some(0),
            player_position: None,
        }
    }

    pub fn reduce(&self, action: &Action) -> State {
        use reducers::*;
        combine_reducers!(self, State, action, {
            my_garden: garden,
            game_tick: game_tick,
            player_position: player_position
        })
    }
}

pub mod selectors;
mod utils;
