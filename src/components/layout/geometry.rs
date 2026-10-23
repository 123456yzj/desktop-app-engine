#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn inset(self, p: f32) -> Self {
        Self {
            x: self.x + p,
            y: self.y + p,
            w: (self.w - 2.0 * p).max(0.0),
            h: (self.h - 2.0 * p).max(0.0),
        }
    }

    pub fn intersect(self, b: Self) -> Self {
        let x = self.x.max(b.x);
        let y = self.y.max(b.y);
        Self {
            x,
            y,
            w: ((self.x + self.w).min(b.x + b.w) - x).max(0.0),
            h: ((self.y + self.h).min(b.y + b.h) - y).max(0.0),
        }
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        self.w > 0.0
            && self.h > 0.0
            && x >= self.x
            && x <= self.x + self.w
            && y >= self.y
            && y <= self.y + self.h
    }

    pub(super) fn rounded(self, scale: f32) -> Self {
        let round = |n: f32| (n * scale).round() / scale;
        Self {
            x: round(self.x),
            y: round(self.y),
            w: round(self.x + self.w) - round(self.x),
            h: round(self.y + self.h) - round(self.y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Limit {
    Bounded(f32),
    Unbounded,
}

#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub width: Limit,
    pub height: Limit,
}
