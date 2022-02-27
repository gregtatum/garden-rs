use std::rc::Rc;

use crate::{game::primitives::Entity, State};
use rltk::{Rltk, RGB};

pub trait Draw {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, entity: &T);
}

#[derive(PartialEq, Debug, Clone)]
pub struct Glyph {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

impl Draw for Glyph {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, entity: &T) {
        let position = entity.position(state);
        ctx.set(position.x, position.y, self.fg, self.bg, self.glyph);
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum LineType {
    Single,
    Double,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Box {
    pub line_type: LineType,
    pub fg: RGB,
    pub bg: RGB,
}

impl Draw for Box {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, entity: &T) {
        let bbox = entity.bbox(state);
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

#[derive(PartialEq, Debug, Clone)]
pub struct Text {
    pub string: String,
    pub fg: RGB,
    pub bg: RGB,
}

impl Draw for Text {
    fn draw<T: Entity>(&self, state: Rc<State>, ctx: &mut Rltk, entity: &T) {
        ctx.print(
            entity.position(state.clone()).x,
            entity.position(state).y,
            &self.string,
        )
    }
}
