use super::{
    drawable::{self, LineType},
    primitives::{BBox, Entity, Position},
};
use rltk::{Rltk, RGB};

/// This is anything needed for rendering a garden entity, which is separate from its
/// serialized form in the blockchain.
#[derive(PartialEq)]
pub struct Garden {
    pub bbox: BBox<i64>,
    pub drawable: drawable::Box,
}

impl Garden {
    pub fn new(bbox: BBox<i64>) -> Self {
        Self {
            bbox,
            drawable: drawable::Box {
                line_type: LineType::Double,
                fg: RGB::named(rltk::BROWN1),
                bg: RGB::named(rltk::BLACK),
            },
        }
    }

    pub fn update(&self) {
        // Do nothing for now.
    }
}

impl drawable::Draw for Garden {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.drawable.draw(ctx, self);
    }
}

impl Entity for Garden {
    fn position<'a>(&'a self) -> Position {
        Position::new(
            self.bbox.top_left.x + (self.bbox.size.x / 2),
            self.bbox.top_left.y + (self.bbox.size.y / 2),
        )
    }

    fn bbox<'a>(&'a self) -> BBox<i64> {
        self.bbox.clone()
    }
}
