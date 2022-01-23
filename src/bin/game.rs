//! This file contains an experimental game client. It needs to be hooked up to
//! everything still.

use garden::game::{
    drawable::{self, Draw, LineType},
    primitives::{BBox, Entity, Position, Size, Vec2},
};
use rltk::{Rltk, RltkBuilder, VirtualKeyCode, RGB};

/// This is anything needed for rendering a garden entity, which is separate from its
/// serialized form in the blockchain.
#[derive(PartialEq)]
pub struct Garden {
    pub bbox: BBox<i64>,
    pub drawable: drawable::Box,
}

impl Garden {
    pub fn new(bbox: BBox<i64>) -> Self {
        Self {
            bbox,
            drawable: drawable::Box {
                line_type: LineType::Double,
                fg: RGB::named(rltk::BROWN1),
                bg: RGB::named(rltk::BLACK),
            },
        }
    }

    pub fn update(&self) {
        // Do nothing for now.
    }
}

impl drawable::Draw for Garden {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.drawable.draw(ctx, self);
    }
}

impl Entity for Garden {
    fn position<'a>(&'a self) -> Position {
        Position::new(
            self.bbox.top_left.x + (self.bbox.size.x / 2),
            self.bbox.top_left.y + (self.bbox.size.y / 2),
        )
    }

    fn bbox<'a>(&'a self) -> BBox<i64> {
        self.bbox.clone()
    }
}

pub struct GameState {
    pub player: Player,
    pub input_device: InputDevice,
    pub gardens: Vec<Garden>,
}

#[derive(PartialEq)]
pub struct Player {
    position: Position,
    glyph: drawable::Glyph,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Position::new(0, 0),
            glyph: drawable::Glyph {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        }
    }

    pub fn update(&mut self, input_device: &InputDevice) {
        self.position += input_device.move_intent;
    }
}

impl drawable::Draw for Player {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.glyph.draw(ctx, self);
    }
}

impl Entity for Player {
    fn position<'a>(&'a self) -> Position {
        self.position
    }
}

pub struct InputDevice {
    move_intent: Position,
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

impl GameState {
    pub fn new() -> Self {
        let bbox = BBox {
            top_left: Position::new(10, 10),
            size: Size::new(30, 20),
        };
        Self {
            player: Player::new(),
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
        self.player.draw(ctx, &self.player);
        for garden in &self.gardens {
            garden.draw(ctx, garden)
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

fn main() -> rltk::BError {
    // Build the terminal.
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let game_state = GameState::new();

    rltk::main_loop(context, game_state)
}
