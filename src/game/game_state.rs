use super::{
    drawable::Draw,
    garden::Garden,
    input_device::InputDevice,
    player::Player,
    primitives::{BBox, Position, Size},
};
use rltk::Rltk;

pub struct GameState {
    pub player: Player,
    pub input_device: InputDevice,
    pub gardens: Vec<Garden>,
}

impl GameState {
    pub fn new() -> Self {
        let bbox = BBox {
            top_left: Position::new(10, 10),
            size: Size::new(30, 20),
        };
        Self {
            player: Player::new(bbox.center()),
            input_device: InputDevice::new(),
            gardens: vec![Garden::new(bbox)],
        }
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.input_device.update(ctx);
        self.player.update(&self.input_device);
        for garden in &self.gardens {
            garden.update();
        }
    }

    pub fn draw(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        for garden in &self.gardens {
            garden.draw(ctx, garden)
        }
        self.player.draw(ctx, &self.player);
    }
}

/// The GameState trait requires the main tick for the program.
impl rltk::GameState for GameState {
    fn tick(&mut self, ctx: &mut Rltk) {
        self.update(ctx);
        self.draw(ctx);
    }
}
