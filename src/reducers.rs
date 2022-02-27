use crate::{
    game::primitives::Position, garden::GardenPlot, Action, ChainAction, GameAction,
};
use std::rc::Rc;

pub fn garden(state: Option<Rc<GardenPlot>>, action: &Action) -> Option<Rc<GardenPlot>> {
    match action {
        Action::Chain(ChainAction::CreatePlot(plot)) => {
            if state.is_some() {
                // Do not allow overriding the garden.
                return state;
            }
            Some(Rc::new(plot.clone()))
        }
        _ => state,
    }
}

pub fn game_tick(state: Option<i64>, action: &Action) -> Option<i64> {
    match action {
        Action::Game(GameAction::TickGame(tick)) => Some(*tick),
        _ => state,
    }
}

pub fn move_intent(
    state: Option<Rc<(Position, i64)>>,
    event: &Action,
) -> Option<Rc<(Position, i64)>> {
    match event {
        Action::Game(GameAction::TickGame(tick)) => None,
        _ => state,
    }
}

pub fn player_position(state: Option<Position>, event: &Action) -> Option<Position> {
    match event {
        Action::Chain(ChainAction::MovePlayer((position, move_intent))) => {
            Some(*position)
        }
        _ => state,
    }
}
