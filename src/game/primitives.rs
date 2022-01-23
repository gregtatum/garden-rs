use num_traits::{FromPrimitive, Num};

pub trait Number: Num + Copy + FromPrimitive {}
impl Number for i64 {}

/// A base trait to be implemented by something in the game.
pub trait Entity: PartialEq {
    fn position(&self) -> Position;
    fn bbox(&self) -> BBox<i64> {
        BBox {
            top_left: self.position(),
            size: Size::new(0, 0),
        }
    }
}

/// I should probably just import a vector library instead of roll
/// my own, but here we are.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2<T: Number> {
    pub x: T,
    pub y: T,
}

pub type Position = Vec2<i64>;
pub type Size = Vec2<i64>;

impl<T: Number> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Number> std::ops::Add<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T>;

    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Number> std::ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl<T: Number> std::ops::Div<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T>;

    fn div(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::new(self.x / other.x, self.y / other.y)
    }
}

impl<T: Number> std::ops::DivAssign for Vec2<T> {
    fn div_assign(&mut self, other: Self) {
        let x = self.x / other.x;
        let y = self.y / other.y;
        *self = Self { x, y };
    }
}

impl<T: Number> std::ops::Div<T> for Vec2<T> {
    type Output = Vec2<T>;

    fn div(self, other: T) -> Vec2<T> {
        Vec2::new(self.x / other, self.y / other)
    }
}

impl<T: Number> From<T> for Vec2<T> {
    fn from(other: T) -> Self {
        Self { x: other, y: other }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BBox<T: Number> {
    pub top_left: Vec2<T>,
    pub size: Vec2<T>,
}

impl<T: Number> BBox<T> {
    pub fn center(&self) -> Vec2<T> {
        self.top_left + (self.size / T::from_u8(2).unwrap())
    }
}
