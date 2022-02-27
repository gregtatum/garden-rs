use std::rc::Rc;

use crate::{selectors, State};

use super::{
    drawable,
    garden::DrawableGarden,
    input_device::InputDevice,
    primitives::{Entity, Position},
    ui,
};
use rltk::{Rltk, RGB};

#[derive(PartialEq)]
pub struct Player {
    pub glyph: drawable::Glyph,
}

impl Player {
    pub fn new() -> Self {
        Self {
            glyph: drawable::Glyph {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        }
    }
}

impl drawable::Draw for Player {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, _entity: &T) {
        self.glyph.draw(state, ctx, self);
    }
}

impl Entity for Player {
    fn position<'a>(&'a self, state: Rc<State>) -> Position {
        selectors::get_player_position(state).expect("Failed to get the player position.")
    }
}
