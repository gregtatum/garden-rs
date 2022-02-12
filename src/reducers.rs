use crate::{garden::GardenPlot, Action};
use std::rc::Rc;

pub fn garden(state: Option<Rc<GardenPlot>>, event: &Action) -> Option<Rc<GardenPlot>> {
    match event {
        Action::CreatePlot(plot) => Some(Rc::new(plot.clone())),
        _ => state,
    }
}
