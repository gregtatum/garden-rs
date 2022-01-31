use super::primitives::{Position, Vec2};
use rltk::{Rltk, VirtualKeyCode};

pub struct InputDevice {
    pub move_intent: Position,
    pub is_enter: bool,
    pub is_backspace: bool,
    pub letter: Option<char>,
}

impl InputDevice {
    pub fn new() -> Self {
        Self {
            move_intent: Vec2::new(0, 0),
            is_enter: false,
            is_backspace: false,
            letter: None,
        }
    }

    pub fn update(&mut self, ctx: &mut Rltk) {
        self.move_intent = Vec2::new(0, 0);
        self.is_enter = false;
        self.is_backspace = false;
        self.letter = None;

        if let Some(key) = ctx.key {
            match key {
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
                VirtualKeyCode::Return => {
                    self.is_enter = true;
                }
                VirtualKeyCode::Back => {
                    self.is_backspace = true;
                }
                _ => {}
            }

            self.letter = if ctx.shift {
                match key {
                    VirtualKeyCode::Key1 => Some('!'),
                    VirtualKeyCode::Key2 => Some('@'),
                    VirtualKeyCode::Key3 => Some('#'),
                    VirtualKeyCode::Key4 => Some('$'),
                    VirtualKeyCode::Key5 => Some('%'),
                    VirtualKeyCode::Key6 => Some('^'),
                    VirtualKeyCode::Key7 => Some('&'),
                    VirtualKeyCode::Key8 => Some('*'),
                    VirtualKeyCode::Key9 => Some('('),
                    VirtualKeyCode::Key0 => Some(')'),
                    VirtualKeyCode::A => Some('A'),
                    VirtualKeyCode::B => Some('B'),
                    VirtualKeyCode::C => Some('C'),
                    VirtualKeyCode::D => Some('D'),
                    VirtualKeyCode::E => Some('E'),
                    VirtualKeyCode::F => Some('F'),
                    VirtualKeyCode::G => Some('G'),
                    VirtualKeyCode::H => Some('H'),
                    VirtualKeyCode::I => Some('I'),
                    VirtualKeyCode::J => Some('J'),
                    VirtualKeyCode::K => Some('K'),
                    VirtualKeyCode::L => Some('L'),
                    VirtualKeyCode::M => Some('M'),
                    VirtualKeyCode::N => Some('N'),
                    VirtualKeyCode::O => Some('O'),
                    VirtualKeyCode::P => Some('P'),
                    VirtualKeyCode::Q => Some('Q'),
                    VirtualKeyCode::R => Some('R'),
                    VirtualKeyCode::S => Some('S'),
                    VirtualKeyCode::T => Some('T'),
                    VirtualKeyCode::U => Some('U'),
                    VirtualKeyCode::V => Some('V'),
                    VirtualKeyCode::W => Some('W'),
                    VirtualKeyCode::X => Some('X'),
                    VirtualKeyCode::Y => Some('Y'),
                    VirtualKeyCode::Z => Some('Z'),

                    VirtualKeyCode::Apostrophe => Some('\''),
                    VirtualKeyCode::Asterisk => Some('*'),
                    VirtualKeyCode::At => Some('@'),
                    VirtualKeyCode::Backslash => Some('\\'),
                    VirtualKeyCode::Colon => Some(':'),
                    VirtualKeyCode::Comma => Some(','),
                    VirtualKeyCode::Equals => Some('='),
                    VirtualKeyCode::Grave => Some('`'),
                    VirtualKeyCode::Minus => Some('-'),
                    VirtualKeyCode::Period => Some('.'),
                    VirtualKeyCode::Plus => Some('+'),
                    VirtualKeyCode::Semicolon => Some(';'),
                    VirtualKeyCode::Slash => Some('/'),
                    VirtualKeyCode::Space => Some(' '),
                    _ => None,
                }
            } else {
                match key {
                    VirtualKeyCode::Key1 => Some('1'),
                    VirtualKeyCode::Key2 => Some('2'),
                    VirtualKeyCode::Key3 => Some('3'),
                    VirtualKeyCode::Key4 => Some('4'),
                    VirtualKeyCode::Key5 => Some('5'),
                    VirtualKeyCode::Key6 => Some('6'),
                    VirtualKeyCode::Key7 => Some('7'),
                    VirtualKeyCode::Key8 => Some('8'),
                    VirtualKeyCode::Key9 => Some('9'),
                    VirtualKeyCode::Key0 => Some('0'),
                    VirtualKeyCode::A => Some('a'),
                    VirtualKeyCode::B => Some('b'),
                    VirtualKeyCode::C => Some('c'),
                    VirtualKeyCode::D => Some('d'),
                    VirtualKeyCode::E => Some('e'),
                    VirtualKeyCode::F => Some('f'),
                    VirtualKeyCode::G => Some('g'),
                    VirtualKeyCode::H => Some('h'),
                    VirtualKeyCode::I => Some('i'),
                    VirtualKeyCode::J => Some('j'),
                    VirtualKeyCode::K => Some('k'),
                    VirtualKeyCode::L => Some('l'),
                    VirtualKeyCode::M => Some('m'),
                    VirtualKeyCode::N => Some('n'),
                    VirtualKeyCode::O => Some('o'),
                    VirtualKeyCode::P => Some('p'),
                    VirtualKeyCode::Q => Some('q'),
                    VirtualKeyCode::R => Some('r'),
                    VirtualKeyCode::S => Some('s'),
                    VirtualKeyCode::T => Some('t'),
                    VirtualKeyCode::U => Some('u'),
                    VirtualKeyCode::V => Some('v'),
                    VirtualKeyCode::W => Some('w'),
                    VirtualKeyCode::X => Some('x'),
                    VirtualKeyCode::Y => Some('y'),
                    VirtualKeyCode::Z => Some('z'),

                    VirtualKeyCode::Apostrophe => Some('\''),
                    VirtualKeyCode::Asterisk => Some('*'),
                    VirtualKeyCode::At => Some('@'),
                    VirtualKeyCode::Backslash => Some('\\'),
                    VirtualKeyCode::Colon => Some(':'),
                    VirtualKeyCode::Comma => Some(','),
                    VirtualKeyCode::Equals => Some('='),
                    VirtualKeyCode::Grave => Some('`'),
                    VirtualKeyCode::Minus => Some('-'),
                    VirtualKeyCode::Period => Some('.'),
                    VirtualKeyCode::Plus => Some('+'),
                    VirtualKeyCode::Semicolon => Some(';'),
                    VirtualKeyCode::Slash => Some('/'),
                    VirtualKeyCode::Space => Some(' '),
                    _ => None,
                }
            }
        }
    }
}
