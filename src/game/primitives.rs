use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::State;

pub trait Number:
    // Define all the traits needed for generic numbers.
    Copy
    + std::cmp::PartialOrd
    + num_traits::Num
    + num_traits::FromPrimitive
{
}
impl Number for i8 {}
impl Number for u8 {}
impl Number for i16 {}
impl Number for u16 {}
impl Number for i32 {}
impl Number for u32 {}
impl Number for i64 {}
impl Number for u64 {}
impl Number for i128 {}
impl Number for u128 {}
impl Number for isize {}
impl Number for usize {}

/// A base trait to be implemented by something in the game.
pub trait Entity: PartialEq {
    fn position(&self, state: Rc<State>) -> Position;
    fn bbox(&self, state: Rc<State>) -> BBox<i32> {
        BBox {
            top_left: self.position(state),
            size: Size::new(0, 0),
        }
    }
}

impl Entity for Position {
    fn position(&self, state: Rc<State>) -> Position {
        self.clone()
    }
}

/// I should probably just import a vector library instead of roll
/// my own, but here we are.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Vec2<T: Number> {
    pub x: T,
    pub y: T,
}

pub type Position = Vec2<i32>;
pub type Size = Vec2<i32>;

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
    pub fn left(&self) -> T {
        self.top_left.x
    }
    pub fn right(&self) -> T {
        self.top_left.x + self.size.x
    }
    pub fn top(&self) -> T {
        self.top_left.y
    }
    pub fn bottom(&self) -> T {
        self.top_left.y + self.size.y
    }
    pub fn intersects_point(&self, point: Vec2<T>) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bbox() {
        let bbox = BBox {
            top_left: Position::new(1, 1),
            size: Size::new(2, 4),
        };
        // Test the corners of the box
        assert!(bbox.intersects_point(Position::new(1 + 0, 1 + 0)));
        assert!(bbox.intersects_point(Position::new(1 + 0, 1 + 4)));
        assert!(bbox.intersects_point(Position::new(1 + 2, 1 + 4)));
        assert!(bbox.intersects_point(Position::new(1 + 2, 1 + 0)));
        assert!(bbox.intersects_point(bbox.center()));

        // Test outside
        assert!(!bbox.intersects_point(Position::new(100, 0)));
        assert!(!bbox.intersects_point(Position::new(0, 100)));
        assert!(!bbox.intersects_point(Position::new(-100, 0)));
        assert!(!bbox.intersects_point(Position::new(0, -100)));

        assert_eq!(bbox.center(), Position::new(2, 3));
    }

    #[test]
    fn test_vec() {
        assert_eq!(
            Vec2::<u8>::new(1, 2) + Vec2::<u8>::new(3, 5),
            Vec2::<u8>::new(4, 7)
        );
    }
}
