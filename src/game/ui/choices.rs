use crate::game::{
    drawable,
    input_device::InputDevice,
    primitives::{BBox, Entity, Position, Size},
};
use rltk::Rltk;

#[derive(PartialEq, Debug, Clone)]
pub struct Choices {
    pub values: Vec<drawable::Text>,
    pub box_: drawable::Box,
    pub cursor: drawable::Glyph,
    pub bbox: BBox<i32>,
    pub cursor_index: i32,
}

impl Choices {
    pub fn new(mut values: Vec<String>) -> Self {
        let mut width = 0;
        for value in &values {
            width = width.max(value.len());
        }
        Self {
            cursor_index: 0,
            box_: drawable::Box {
                line_type: drawable::LineType::Single,
                fg: rltk::GRAY60.into(),
                bg: rltk::BLACK.into(),
            },
            bbox: BBox {
                top_left: Position::new(0, 0),
                size: Size::new(width as i32 + 4, values.len() as i32 + 1),
            },
            // Convert into drawable::Text
            values: values
                .drain(0..values.len())
                .map(|string| drawable::Text {
                    string,
                    fg: rltk::WHITE.into(),
                    bg: rltk::BLACK.into(),
                })
                .collect(),
            cursor: drawable::Glyph {
                glyph: rltk::to_cp437('○'),
                fg: rltk::WHITE.into(),
                bg: rltk::BLACK.into(),
            },
        }
    }

    pub fn update(&mut self, input_device: &InputDevice) -> Option<String> {
        let len = self.values.len() as i32;
        self.cursor_index =
            ((self.cursor_index as i32 + input_device.move_intent.y) + len) % len;
        if input_device.is_enter {
            Some(
                self.values
                    .get(self.cursor_index as usize)
                    .expect("Unable to get value.")
                    .string
                    .clone(),
            )
        } else {
            None
        }
    }

    /// Centers the text input in a given window size.
    pub fn center(&mut self, w: i32, h: i32) {
        self.bbox.top_left.x = (w - self.bbox.size.x) / 2;
        self.bbox.top_left.y = (h - self.bbox.size.y) / 2;
    }
}

impl drawable::Draw for Choices {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.box_.draw(ctx, self);
        for (i, value) in self.values.iter().enumerate() {
            value.draw(
                ctx,
                &Position::new(
                    self.bbox.top_left.x + 3,
                    self.bbox.top_left.y + 1 + i as i32,
                ),
            );
        }
        self.cursor.draw(
            ctx,
            &Position::new(
                self.bbox.top_left.x + 1,
                self.bbox.top_left.y + 1 + self.cursor_index,
            ),
        );
    }
}

impl Entity for Choices {
    fn position<'a>(&'a self) -> Position {
        Position::new(
            self.bbox.top_left.x + (self.bbox.size.x / 2),
            self.bbox.top_left.y + (self.bbox.size.y / 2),
        )
    }

    fn bbox<'a>(&'a self) -> BBox<i32> {
        self.bbox.clone()
    }
}
