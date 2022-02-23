use crate::{garden::GardenPlot, Action};
use std::rc::Rc;

pub fn garden(
    state: Rc<Option<Rc<GardenPlot>>>,
    event: &Action,
) -> Rc<Option<Rc<GardenPlot>>> {
    match event {
        Action::CreatePlot(plot) => {
            if state.is_some() {
                // Do not allow overriding the garden.
                return state;
            }
            Rc::from(Some(Rc::new(plot.clone())))
        }
        #[allow(unused)] // Will be used.
        _ => state,
    }
}
