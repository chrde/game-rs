use super::*;

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
        test.x() >= self.min.x() && test.y() >= self.min.y() && test.x() < self.max.x() && test.y() < self.max.y()
    }
}
