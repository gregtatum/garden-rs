use crate::game::primitives::Entity;
use rltk::{Rltk, RGB};

pub trait Draw {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, entity: &T);
}

#[derive(PartialEq)]
pub struct Glyph {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

impl Draw for Glyph {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, entity: &T) {
        let position = entity.position();
        ctx.set(position.x, position.y, self.fg, self.bg, self.glyph);
    }
}

#[derive(PartialEq)]
pub enum LineType {
    Single,
    Double,
}

#[derive(PartialEq)]
pub struct Box {
    pub line_type: LineType,
    pub fg: RGB,
    pub bg: RGB,
}

impl Draw for Box {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, entity: &T) {
        let bbox = entity.bbox();
        ctx.draw_hollow_box_double(
            bbox.top_left.x,
            bbox.top_left.y,
            bbox.size.x,
            bbox.size.y,
            self.fg,
            self.bg,
        )
    }
}