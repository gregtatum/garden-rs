use uuid::Uuid;

pub struct GardenPlot {
    pub uuid: Uuid,
}

impl GardenPlot {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}
