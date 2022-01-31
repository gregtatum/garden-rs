use crate::garden::TheLand;

use super::{
    drawable::Draw,
    garden::Garden,
    input_device::InputDevice,
    player::Player,
    primitives::{BBox, Position, Size},
    ui,
};
use rltk::Rltk;

pub struct GameState {
    player: Player,
    input_device: InputDevice,
    gardens: Vec<Garden>,
    input_ui: Option<ui::InputUI>,
    input_handler: ui::InputHandler,
    the_land: TheLand,
}

const GAME_W: i32 = 80;
const GAME_H: i32 = 50;

impl GameState {
    pub fn new() -> Self {
        let mut game_state = Self {
            player: Player::new(Position::new(-1, -1)),
            input_device: InputDevice::new(),
            gardens: vec![],
            input_ui: None,
            input_handler: Default::default(),
            the_land: TheLand::new(),
        };

        if game_state.gardens.is_empty() {
            game_state.ask_new_garden()
        }

        game_state
    }

    pub fn ask_new_garden(&mut self) {
        let mut text_input = ui::TextInput::new(String::from(""), 30);
        text_input.center(GAME_W, GAME_H);
        self.input_ui = Some(ui::InputUI::TextInput(text_input));
        self.input_handler = ui::InputHandler::NewGarden;
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.input_device.update(ctx);
        self.player
            .update(&self.input_device, &self.gardens, &self.input_ui);
        for garden in &self.gardens {
            garden.update();
        }
        if let Some(ref mut input_ui) = self.input_ui {
            if let Some(text) = match input_ui {
                ui::InputUI::Choices(input) => input.update(&self.input_device),
                ui::InputUI::TextInput(input) => input.update(&self.input_device, &ctx),
            } {
                self.input_ui = None;
                self.handle_input(text);
            }
        }
    }

    pub fn handle_input(&mut self, text: String) {
        match self.input_handler {
            ui::InputHandler::NewGarden => {
                let (hash, plot) = self.the_land.create_garden_plot(text);
                let margin = 10;
                let bbox = BBox {
                    top_left: Position::new(margin, margin),
                    size: Size::new(GAME_W - margin * 2, GAME_H - margin * 2),
                };
                if self.gardens.len() == 0 {
                    // This is the first garden, place the player in it.
                    self.player.position = bbox.center();
                }
                self.gardens.push(Garden::new(bbox, hash, plot));
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        for garden in &self.gardens {
            garden.draw(ctx, garden)
        }
        self.player.draw(ctx, &self.player);
        if let Some(ref input_ui) = self.input_ui {
            match input_ui {
                ui::InputUI::Choices(input) => input.draw(ctx, input),
                ui::InputUI::TextInput(input) => input.draw(ctx, input),
            }
        }
    }
}

/// The GameState trait requires the main tick for the program.
impl rltk::GameState for GameState {
    fn tick(&mut self, ctx: &mut Rltk) {
        self.update(ctx);
        self.draw(ctx);
    }
}
