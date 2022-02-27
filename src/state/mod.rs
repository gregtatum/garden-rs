use crate::{
    combine_reducers, game::primitives::Position, garden::GardenPlot, reducers, Action,
};
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub struct State {
    my_garden: Option<Rc<GardenPlot>>,
    game_tick: Option<i64>,
    move_intent: Option<(Position, i64)>,
}

impl State {
    pub fn new() -> Self {
        Self {
            my_garden: None,
            game_tick: Some(0),
            move_intent: None,
        }
    }

    pub fn reduce(&self, action: &Action) -> State {
        use reducers::*;
        combine_reducers!(State, action, {
            my_garden: garden,
            game_tick: game_tick,
            move_intent: move_intent,
        })
    }
}

pub mod selectors;
mod utils;
