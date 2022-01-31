use super::{
    drawable,
    garden::Garden,
    input_device::InputDevice,
    primitives::{Entity, Position},
    ui,
};
use rltk::{Rltk, RGB};

#[derive(PartialEq)]
pub struct Player {
    pub position: Position,
    pub glyph: drawable::Glyph,
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

    pub fn update(
        &mut self,
        input_device: &InputDevice,
        gardens: &Vec<Garden>,
        input: &Option<ui::InputUI>,
    ) {
        if input.is_some() {
            // Do not move the player if there is some input to be made.
            return;
        }
        let mut next_position = self.position + input_device.move_intent;
        for garden in gardens {
            if garden.bbox.intersects_point(self.position) {
                if garden.bbox.left() == next_position.x
                    || garden.bbox.right() == next_position.x
                    || garden.bbox.top() == next_position.y
                    || garden.bbox.bottom() == next_position.y
                {
                    next_position = self.position;
                    break;
                }
            }
        }
        self.position = next_position;
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
