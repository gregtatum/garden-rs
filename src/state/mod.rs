use crate::{garden::GardenPlot, reducers, Action};
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub struct State {
    my_garden: Option<Rc<GardenPlot>>,
}

impl State {
    pub fn new() -> Self {
        Self { my_garden: None }
    }

    pub fn reduce(&self, event: &Action) -> State {
        State {
            my_garden: reducers::garden(self.my_garden.clone(), event),
        }
    }
}

pub mod selectors;
