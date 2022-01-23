use num_traits::Num;

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2<T: Num + Copy> {
    pub x: T,
    pub y: T,
}

pub type Position = Vec2<i64>;
pub type Size = Vec2<i64>;

impl<T: Num + Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Num + Copy> std::ops::Add<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T>;

    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Num + Copy> std::ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BBox<T: Num + Copy> {
    pub top_left: Vec2<T>,
    pub size: Vec2<T>,
}
