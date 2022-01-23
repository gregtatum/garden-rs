use super::{
    drawable,
    input_device::InputDevice,
    primitives::{Entity, Position},
};
use rltk::{Rltk, RGB};

#[derive(PartialEq)]
pub struct Player {
    position: Position,
    glyph: drawable::Glyph,
}

impl Player {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            glyph: drawable::Glyph {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        }
    }

    pub fn update(&mut self, input_device: &InputDevice) {
        self.position += input_device.move_intent;
    }
}

impl drawable::Draw for Player {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.glyph.draw(ctx, self);
    }
}

impl Entity for Player {
    fn position<'a>(&'a self) -> Position {
        self.position
    }
}
