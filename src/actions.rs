use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{block_chain::SerializedBytes, garden::GardenPlot};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Action {
    CreatePlot(GardenPlot),
}

impl SerializedBytes for Action {
    fn serialized_bytes(&self) -> Cow<[u8]> {
        Cow::from(bincode::serialize(self).expect("Unable to serialize Action."))
    }
}

pub fn create_garden_plot(name: String) -> Action {
    Action::CreatePlot(GardenPlot::new(name))
}
