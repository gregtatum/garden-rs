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

pub fn get_my_garden(state: Rc<State>) -> Rc<Option<Rc<GardenPlot>>> {
    state.my_garden.clone()
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
    ({
        name: $fn_name:ident,
        returns: $Returns:ty,
        selectors: {
            $(
                $selector_fn_name:ident:
                ($selector_var_name:ident: $SelectorReturns:ty)
            ),*
        },
        contents: $contents:tt
    }) => { paste! {
        pub fn $fn_name(state: Rc<State>) -> $Returns {
            // Call out to the $fn_name_selector_impl module for proper macro hygiene.
            [<$fn_name _selector_impl>]::$fn_name(state)
        }

        mod [<$fn_name _selector_impl>] {
            use super::*;
            use std::rc::Rc;
            use std::option::Option;
            use crate::State;
            use std::borrow::Borrow;

            // e.g. the tuple: (ArgTypeA, ArgTypeB)
            type ArgsCacheTuple = ( $( $SelectorReturns ),* );

            thread_local! {
                // The arguments cache:
                // ```
                // pub static SELECTOR_GET_STATE_ARGS_CACHE: RefCell<Option<(Arg1, Arg2)>>
                //     = RefCell::new(None);
                // ```
                pub static [<SELECTOR_ $fn_name:upper _ARGS_CACHE>]: RefCell<Option<ArgsCacheTuple>> = RefCell::new(None);

                // The returns cache:
                // ```
                // pub static SELECTOR_GET_STATE_RETURNS_CACHE: RefCell<Option<ReturnType>>
                //     = RefCell::new(None);
                // ```
                pub static [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>]: RefCell<Option<$Returns>> = RefCell::new(None);
            }

            #[inline]
            pub fn $fn_name(state: Rc<State>) -> $Returns {
                // Name gross these gross invocations.
                let ref args_cache = [<SELECTOR_ $fn_name:upper _ARGS_CACHE>];
                let ref returns_cache = [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>];

                // Get the selector values.
                // let value_a = get_value_b(state.clone());
                // let value_b = get_value_b(state.clone());
                $(
                    let $selector_var_name = $selector_fn_name(state.clone());
                )*

                // See if the cached args match.
                let mut cache_matches = true;
                args_cache.with(|f| {
                    // Get each variable out of the cache.
                    if let Some((
                        $( ref [<cached_ $selector_var_name>] ),*
                    )) = *f.borrow() {
                        $(
                            // Check pointer equality.
                            if cache_matches && !std::rc::Rc::ptr_eq(&$selector_var_name, [<cached_ $selector_var_name>]) {
                                cache_matches = false;
                            }
                        )*
                    } else {
                        cache_matches = false;
                    }

                    if !cache_matches {
                        // Update the cached args by cloning the Rc.
                        let new_cache: ArgsCacheTuple = (
                            $( $selector_var_name.clone() ),*
                        );
                        *f.borrow_mut() = Some(new_cache);
                    }
                });

                // This is a cache hit, return from the cache.
                if cache_matches {
                    let mut result: Option<$Returns> = None;
                    returns_cache.with(|f| {
                        result = (*f.borrow()).clone();
                    });
                    return result.expect("Logic error, failed to get returns from cache.");
                }

                let return_value: $Returns = [<selector_ $fn_name _impl>]($( $selector_var_name ),*);

                let ref returns_cache = [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>];
                returns_cache.with(|f| {
                    *f.borrow_mut() = Some(return_value.clone());
                });

                return_value
            }

            pub fn [<selector_ $fn_name _impl>](
                $( $selector_var_name: $SelectorReturns ),*
            ) -> $Returns {
                $contents
            }
        }
    }};
}

selector!({
    name: get_drawable_garden,
    returns: Rc<Option<DrawableGarden>>,
    selectors: {
        get_my_garden: (plot: Rc<Option<Rc<GardenPlot>>>)
    },
    contents: {
        if let Some(ref plot) = *plot {
            println!("has plot");
            let margin = 10;
            let bbox = BBox {
                top_left: Position::new(margin, margin),
                size: Size::new(GAME_W - margin * 2, GAME_W - margin * 2),
            };
            let todo = Hash::empty();
            return Rc::from(Some(DrawableGarden::new(bbox, todo, plot.clone())));
        }
        Rc::from(None)
    }
});
