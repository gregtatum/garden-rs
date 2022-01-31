use crate::{
    game::{
        drawable::{self, LineType},
        primitives::{BBox, Entity, Position},
    },
    garden::GardenPlot,
    hash::Hash,
};

use rltk::{Rltk, RGB};

/// This is anything needed for rendering a garden entity, which is separate from its
/// serialized form in the blockchain.
#[derive(PartialEq)]
pub struct Garden {
    pub bbox: BBox<i32>,
    pub drawable_box: drawable::Box,
    pub drawable_text: drawable::Text,
    pub plot: GardenPlot,
    pub hash: Hash,
}

impl Garden {
    pub fn new(bbox: BBox<i32>, hash: Hash, plot: GardenPlot) -> Self {
        let fg = RGB::named(rltk::BROWN1);
        let bg = RGB::named(rltk::BLACK);
        Self {
            bbox,
            drawable_box: drawable::Box {
                line_type: LineType::Double,
                fg,
                bg,
            },
            drawable_text: drawable::Text {
                string: format!("<{}>", plot.name),
                fg,
                bg,
            },
            hash,
            plot,
        }
    }

    pub fn update(&self) {
        // Do nothing for now.
    }
}

impl drawable::Draw for Garden {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.drawable_box.draw(ctx, self);
        let mut position = self.bbox.top_left;
        position.x += 2;
        self.drawable_text.draw(ctx, &position);
    }
}

impl Entity for Garden {
    fn position<'a>(&'a self) -> Position {
        Position::new(
            self.bbox.top_left.x + (self.bbox.size.x / 2),
            self.bbox.top_left.y + (self.bbox.size.y / 2),
        )
    }

    fn bbox<'a>(&'a self) -> BBox<i32> {
        self.bbox.clone()
    }
}
