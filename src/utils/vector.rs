use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2d {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Vec2d {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from_angle(a: f64) -> Self {
        Self::new(a.cos(), a.sin())
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::ZERO } else { self / len }
    }

    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn perp_dot(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }
}

impl Add for Vec2d {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2d {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2d {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2d> for f64 {
    type Output = Vec2d;
    fn mul(self, rhs: Vec2d) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vec2d {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2d {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

use glam::Vec2;

impl From<Vec2> for Vec2d {
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
        }
    }
}
