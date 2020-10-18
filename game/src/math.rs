use std::ops::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct V2 {
    x: f32,
    y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn min(&self) -> f32 {
        self.x
    }

    pub fn max(&self) -> f32 {
        self.y
    }

    pub fn inner(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn len(self) -> f32 {
        Self::inner(self, self)
    }

    pub fn rev(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }
}

impl Mul<f32> for V2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for V2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<V2> for f32 {
    type Output = V2;

    fn mul(self, rhs: V2) -> Self::Output {
        V2 {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl Sub for V2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add for V2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl Neg for V2 {
    type Output = V2;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
