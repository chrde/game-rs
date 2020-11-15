use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Rect2 {
    min: V2,
    max: V2,
}

impl Rect2 {
    pub fn new_min_max(min: V2, max: V2) -> Self {
        Self { min, max }
    }

    pub fn new_min_dmin(min: V2, dim: V2) -> Self {
        Self {
            min,
            max: min + dim,
        }
    }

    pub fn new_center_half_dim(center: V2, half_dim: V2) -> Self {
        Self {
            min: center - half_dim,
            max: center + half_dim,
        }
    }

    pub fn new_center_dim(center: V2, dim: V2) -> Self {
        Self::new_center_half_dim(center, dim * 0.5)
    }

    pub fn contains(&self, test: V2) -> bool {
        test.x() >= self.min.x()
            && test.y() >= self.min.y()
            && test.x() < self.max.x()
            && test.y() < self.max.y()
    }

    pub fn center(&self) -> V2 {
        0.5 * (self.min + self.max)
    }

    pub fn add_radius(&self, radius: V2) -> Self {
        Self {
            min: self.min - radius,
            max: self.max + radius,
        }
    }

    pub fn min(&self) -> V2 {
        self.min
    }

    pub fn max(&self) -> V2 {
        self.max
    }
}
