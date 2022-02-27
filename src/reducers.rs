use crate::{game::primitives::Position, garden::GardenPlot, Action};
use std::rc::Rc;

pub fn garden(state: Option<Rc<GardenPlot>>, event: &Action) -> Option<Rc<GardenPlot>> {
    match event {
        Action::CreatePlot(plot) => {
            if state.is_some() {
                // Do not allow overriding the garden.
                return state;
            }
            Some(Rc::new(plot.clone()))
        }
        _ => state,
    }
}

pub fn player_position(
    state: Option<Rc<Position>>,
    event: &Action,
) -> Option<Rc<Position>> {
    match event {
        Action::MovePlayer(position) => Some(Rc::new(*position)),
        _ => state,
    }
}

pub fn game_tick(state: i64, event: &Action) -> i64 {
    match event {
        Action::TickGame(tick) => *tick,
        _ => state,
    }
}
