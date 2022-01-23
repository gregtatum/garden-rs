use super::primitives::{Position, Vec2};
use rltk::{Rltk, VirtualKeyCode};

pub struct InputDevice {
    pub move_intent: Position,
}

impl InputDevice {
    pub fn new() -> Self {
        Self {
            move_intent: Vec2::new(0, 0),
        }
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.move_intent = Vec2::new(0, 0);
        match ctx.key {
            None => {} // Nothing happened
            Some(key) => match key {
                VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::A => {
                    self.move_intent.x = -1;
                }
                VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::D => {
                    self.move_intent.x = 1;
                }
                VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::W => {
                    self.move_intent.y = -1;
                }
                VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::S => {
                    self.move_intent.y = 1;
                }
                _ => {}
            },
        }
    }
}
