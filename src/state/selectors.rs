use crate::{
    game::{
        drawable::{self, LineType},
        game_state::{GAME_H, GAME_W},
        garden::DrawableGarden,
        player::Player,
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
            eprintln!("Generating empty hash.");
            let todo = Hash::empty();
            return Some(Rc::from(DrawableGarden::new(GardenPlot::get_default_bbox(), todo, plot.clone())));
            }
            None
        }
    }
);

fn get_player_is_some(state: Rc<State>) -> bool {
    state.player_position.is_some()
}

selector!(
    pub fn get_drawable_player(state: Rc<State>) -> Option<Rc<Player>> {
        memoize |
            is_player: get_player_is_some -> bool
        | {
            if is_player {
                Some(Rc::new(Player::new()))
            } else {
                None
            }
        }
    }
);

selector!(
    pub fn get_drawable_gardens(state: Rc<State>) -> Rc<Vec<Rc<DrawableGarden>>> {
        memoize |my_garden: get_drawable_garden -> Option<Rc<DrawableGarden>>| {
            let mut gardens = vec![];
            if let Some(ref my_garden) = my_garden {
                gardens.push(my_garden.clone());
            }
            Rc::from(gardens)
        }
    }
);

pub fn get_player_position(state: Rc<State>) -> Option<Position> {
    state.player_position
}
