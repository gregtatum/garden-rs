//! This file contains an experimental game client. It needs to be hooked up to
//! everything still.

use rltk::{Rltk, RltkBuilder, VirtualKeyCode, RGB};

pub struct Glyph {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

trait Entity {
    fn position<'a>(&'a self) -> &'a Position;
}

trait Draw {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, entity: &T);
}

impl Draw for Glyph {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, entity: &T) {
        ctx.set(
            entity.position().x,
            entity.position().y,
            self.fg,
            self.bg,
            self.glyph,
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add<Position> for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::AddAssign for Position {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

struct GameState {
    player: Player,
    input_device: InputDevice,
}

pub struct Player {
    position: Position,
    glyph: Glyph,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Position::new(0, 0),
            glyph: Glyph {
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

impl Draw for Player {
    fn draw<T: Entity>(&self, ctx: &mut Rltk, _entity: &T) {
        self.glyph.draw(ctx, self);
    }
}

impl Entity for Player {
    fn position<'a>(&'a self) -> &'a Position {
        &self.position
    }
}

pub struct InputDevice {
    move_intent: Position,
}

impl InputDevice {
    pub fn new() -> Self {
        Self {
            move_intent: Position::new(0, 0),
        }
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.move_intent = Position::new(0, 0);
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
        Self {
            player: Player::new(),
            input_device: InputDevice::new(),
        }
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.input_device.update(ctx);
        self.player.update(&self.input_device);
    }

    pub fn draw(&mut self, ctx: &mut Rltk) {
        ctx.cls();
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

fn main() -> rltk::BError {
    // Build the terminal.
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let game_state = GameState::new();

    rltk::main_loop(context, game_state)
}
