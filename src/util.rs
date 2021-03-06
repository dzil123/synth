use std::panic::Location;

pub const BITRATE: u32 = 44100;
pub const BITRATE_F: f32 = BITRATE as _;

// (0 < x < 1) to (a < ans < b)
pub fn lerp(x: f32, a: f32, b: f32) -> f32 {
    a + x * (b - a)
}

// scale (-1 < x < 1) to (a < ans < b)
pub fn scale(x: f32, a: f32, b: f32) -> f32 {
    (b + lerp(x, a, b)) / 2.0
}

pub fn clamp(x: f32) -> f32 {
    x.min(1.0).max(-1.0)
}

pub fn clamp01(x: f32) -> f32 {
    x.min(1.0).max(0.0)
}

pub fn distort(x: f32, a: f32) -> f32 {
    // clamp(x * (1.0 + a)) // 0 < a < inf
    clamp(x / (1.0 - a)) // 0 < a < 1
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Index {
    Location(Location<'static>),
    Num(usize),
}

impl Index {
    #[track_caller]
    pub fn location() -> Self {
        Location::caller().into()
    }
}

impl From<&'static Location<'static>> for Index {
    fn from(loc: &'static Location<'static>) -> Self {
        Self::Location(*loc)
    }
}

impl From<usize> for Index {
    fn from(num: usize) -> Self {
        Self::Num(num)
    }
}
