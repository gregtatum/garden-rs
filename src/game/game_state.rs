use std::rc::Rc;

use crate::{
    actions, chain_store::ChainStore, selectors, Action, ChainAction, State, Store,
};
use anyhow::Result;

use super::{
    drawable::Draw,
    input_device::InputDevice,
    player::Player,
    primitives::{BBox, Position, Size},
    ui,
};
use rltk::Rltk;

pub enum Phase {
    Playing,
    Menu,
}

pub struct GameState {
    player: Player,
    input_device: InputDevice,
    input_ui: Option<ui::InputUI>,
    input_handler: ui::InputHandler,
    state_store: Store,
    prev_state: Rc<State>,
}

pub const GAME_W: i32 = 80;
pub const GAME_H: i32 = 50;

impl GameState {
    pub fn try_new(chain_store: Box<dyn ChainStore<ChainAction>>) -> Result<Self> {
        let mut game_state = Self {
            player: Player::new(),
            input_device: InputDevice::new(),
            input_ui: None,
            input_handler: Default::default(),
            state_store: Store::try_new(chain_store)?,
            prev_state: Rc::new(State::new()),
        };

        if selectors::get_my_garden(game_state.state()).is_none() {
            game_state.ask_new_garden()
        }

        Ok(game_state)
    }

    pub fn state(&self) -> Rc<State> {
        self.state_store.state.clone()
    }

    pub fn ask_new_garden(&mut self) {
        let mut text_input = ui::TextInput::new(String::from(""), 30);
        text_input.center(GAME_W, GAME_H);
        self.input_ui = Some(ui::InputUI::TextInput(text_input));
        self.input_handler = ui::InputHandler::NewGarden;
    }

    pub fn show_main_menu(&mut self) {
        let mut choices =
            ui::Choices::new(vec![String::from("Save"), String::from("Exit")]);
        choices.center(GAME_W, GAME_H);
        self.input_ui = Some(ui::InputUI::Choices(choices));
        self.input_handler = ui::InputHandler::MainMenu;
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.input_device.update(ctx);
        if self.input_ui.is_none() && self.input_device.is_esc {
            eprintln!("show_main_menu");
            self.show_main_menu();
        }
        self.player
            .update(&self.input_device, &vec![], &self.input_ui);
        // for garden in &self.gardens {
        //     garden.update();
        // }
        if let Some(ref mut input_ui) = self.input_ui {
            if let Some(text) = match input_ui {
                ui::InputUI::Choices(input) => input.update(&self.input_device),
                ui::InputUI::TextInput(input) => input.update(&self.input_device, &ctx),
            } {
                self.input_ui = None;
                self.handle_input(text, ctx);
            }
        }
    }

    pub fn handle_input(&mut self, text: String, ctx: &mut Rltk) {
        match self.input_handler {
            ui::InputHandler::NewGarden => {
                self.state_store.dispatch(actions::create_garden_plot(text));
            }
            ui::InputHandler::MainMenu => {
                if text == "Save" {
                    self.state_store
                        .chains
                        .persist()
                        .expect("Failed to store the block chain");
                } else if text == "Exit" {
                    ctx.quit();
                }
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        if let Some(my_garden) = selectors::get_drawable_garden(self.state()) {
            my_garden.draw(ctx, &*my_garden);
        }
        // for garden in &self.gardens {
        //     garden.draw(ctx, garden)
        // }
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
        if ctx.quitting {
            eprintln!("Quitting");
        }
        self.update(ctx);
        self.draw(ctx);
        self.prev_state = self.state();
    }
}
