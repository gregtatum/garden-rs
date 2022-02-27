use crate::{
    game::{
        drawable::{self, LineType},
        game_state::{GAME_H, GAME_W},
        garden::DrawableGarden,
        primitives::{BBox, Entity, Position, Size},
    },
    garden::GardenPlot,
    selector, Hash, State,
};
use rltk::{Rltk, RGB};
use std::{cell::RefCell, rc::Rc};

pub fn get_my_garden(state: Rc<State>) -> Option<Rc<GardenPlot>> {
    state.my_garden.clone()
}

pub fn get_game_tick(state: Rc<State>) -> Option<i64> {
    state.game_tick
}

selector!(
    pub fn get_drawable_garden(state: Rc<State>) -> Option<Rc<DrawableGarden>> {
        memoize |plot: get_my_garden -> Option<Rc<GardenPlot>>| {
            if let Some(ref plot) = plot {
            let margin = 10;
            let bbox = BBox {
                top_left: Position::new(margin, margin),
                size: Size::new(GAME_W - margin * 2, GAME_H - margin * 2),
            };
            let todo = Hash::empty();
            println!("{:?}", bbox);
            return Some(Rc::from(DrawableGarden::new(bbox, todo, plot.clone())));
            }
            None
        }
    }
);

selector!(
    pub fn get_move_intent(state: Rc<State>) -> Option<Position> {
        memoize |
            game_tick: get_game_tick -> i64
        | {
            if game_tick == state.move_intent.1 {
                if
                Some(state.move_intent.0)
        }
    }
);

selector!(
    pub fn get_player_position(state: Rc<State>) -> Option<Position> {
        memoize |
            drawable_garden: get_drawable_garden -> Option<Rc<DrawableGarden>>,
            game_tick: get_game_tick -> i64,
            move_intent: get_game_tick -> i64,

        | {
            if let Some(drawable_garden) = drawable_garden {
                return Some(drawable_garden.bbox.center());
            }
            None
        }
    }
);
