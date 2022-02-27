use std::rc::Rc;

use rltk::Rltk;

use crate::{
    game::{
        drawable,
        input_device::InputDevice,
        primitives::{BBox, Entity, Position, Size},
    },
    State,
};

#[derive(PartialEq, Debug, Clone)]
pub struct TextInput {
    pub text: drawable::Text,
    pub box_: drawable::Box,
    pub cursor: drawable::Glyph,
    pub bbox: BBox<i32>,
    pub max_width: i32,
    pub blink_time: f32,
}

const BLINK: f32 = 1000.0;

impl TextInput {
    pub fn new(string: String, max_width: i32) -> Self {
        Self {
            max_width,
            box_: drawable::Box {
                line_type: drawable::LineType::Single,
                fg: rltk::GRAY60.into(),
                bg: rltk::BLACK.into(),
            },
            bbox: BBox {
                top_left: Position::new(0, 0),
                size: Size::new(max_width + 3, 4),
            },
            // Convert into drawable::Text
            text: drawable::Text {
                string,
                fg: rltk::WHITE.into(),
                bg: rltk::BLACK.into(),
            },
            cursor: drawable::Glyph {
                glyph: rltk::to_cp437('|'),
                fg: rltk::WHITE.into(),
                bg: rltk::BLACK.into(),
            },
            blink_time: 0.0,
        }
    }

    /// Centers the text input in a given window size.
    pub fn center(&mut self, w: i32, h: i32) {
        self.bbox.top_left.x = (w - self.bbox.size.x) / 2;
        self.bbox.top_left.y = (h - self.bbox.size.y) / 2;
    }

    pub fn update(&mut self, input_device: &InputDevice, ctx: &Rltk) -> Option<String> {
        if let Some(letter) = input_device.letter {
            if self.text.string.len() < self.max_width as usize {
                self.text.string.push(letter);
            }
        }

        self.blink_time = (self.blink_time + ctx.frame_time_ms) % BLINK;

        if input_device.is_backspace {
            self.text.string.pop();
        }

        if input_device.is_enter {
            Some(self.text.string.clone())
        } else {
            None
        }
    }
}

impl drawable::Draw for TextInput {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, _entity: &T) {
        self.box_.draw(state.clone(), ctx, self);
        self.text.draw(
            state.clone(),
            ctx,
            &Position::new(self.bbox.top_left.x + 2, self.bbox.top_left.y + 2),
        );
        if self.blink_time < BLINK / 2.0 {
            self.cursor.draw(
                state,
                ctx,
                &Position::new(
                    self.bbox.top_left.x + 2 + self.text.string.len() as i32,
                    self.bbox.top_left.y + 2,
                ),
            );
        }
    }
}

impl Entity for TextInput {
    fn position<'a>(&'a self, _state: Rc<State>) -> Position {
        Position::new(
            self.bbox.top_left.x + (self.bbox.size.x / 2),
            self.bbox.top_left.y + (self.bbox.size.y / 2),
        )
    }

    fn bbox<'a>(&'a self, _state: Rc<State>) -> BBox<i32> {
        self.bbox.clone()
    }
}
