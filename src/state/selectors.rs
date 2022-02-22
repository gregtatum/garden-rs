use crate::{
    game::{
        drawable::{self, LineType},
        game_state::GAME_W,
        primitives::{BBox, Entity, Position, Size},
    },
    garden::GardenPlot,
    Hash, State,
};
use paste::paste;
use rltk::{Rltk, RGB};
use std::{cell::RefCell, rc::Rc};

pub fn get_my_garden(state: &State) -> &Option<Rc<GardenPlot>> {
    &state.my_garden
}

/// This is anything needed for rendering a garden entity, which is separate from its
/// serialized form in the blockchain.
#[derive(PartialEq)]
pub struct DrawableGarden {
    pub bbox: BBox<i32>,
    pub drawable_box: drawable::Box,
    pub drawable_text: drawable::Text,
    pub plot: Rc<GardenPlot>,
    pub hash: Hash,
}

impl DrawableGarden {
    pub fn new(bbox: BBox<i32>, hash: Hash, plot: Rc<GardenPlot>) -> Self {
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

impl drawable::Draw for DrawableGarden {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.drawable_box.draw(ctx, self);
        let mut position = self.bbox.top_left;
        position.x += 2;
        self.drawable_text.draw(ctx, &position);
    }
}

impl Entity for DrawableGarden {
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

macro_rules! selector {
    ($func_name:ident, $returns:ty, [ $( $args:ident ),* ], $lambda:tt) => {
        paste! {
            thread_local! {
                pub static [<$func_name:upper _RETURNS>]: RefCell<Option<$returns>> = RefCell::new(None);
            }
        }

        // The macro will expand into the contents of this block.
        pub fn $func_name(state: &State) -> $returns {
            let callback = $lambda;
            callback($($args(state))*)
        }
    };
}

selector!(
    get_drawable_garden,
    Option<DrawableGarden>,
    [get_my_garden],
    {
        |plot: &Option<Rc<GardenPlot>>| {
            if let Some(ref plot) = plot {
                let margin = 10;
                let bbox = BBox {
                    top_left: Position::new(margin, margin),
                    size: Size::new(GAME_W - margin * 2, GAME_W - margin * 2),
                };
                let todo = Hash::empty();
                return Some(DrawableGarden::new(bbox, todo, plot.clone()));
            }
            None
        }
    }
);

// selector!({
//     name: get_drawable_garden,
//     returns: Option<DrawableGarden>,
//     selectors: {
//         get_my_garden: (plot: &Option<Rc<GardenPlot>>)
//     },
//     contents: {
//         if let Some(ref plot) = plot {
//             let margin = 10;
//             let bbox = BBox {
//                 top_left: Position::new(margin, margin),
//                 size: Size::new(GAME_W - margin * 2, GAME_W - margin * 2),
//             };
//             let todo = Hash::empty();
//             return Some(DrawableGarden::new(bbox, todo, plot.clone()));
//         }
//         None
//     }
// });
