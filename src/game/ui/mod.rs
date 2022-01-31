mod choices;
mod text_input;

pub use choices::Choices;
pub use text_input::TextInput;

pub enum InputUI {
    Choices(Choices),
    TextInput(TextInput),
}

pub enum InputHandler {
    NewGarden,
}

impl Default for InputHandler {
    fn default() -> Self {
        // Arbitrarily pick one.
        InputHandler::NewGarden
    }
}
