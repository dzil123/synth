use std::any::Any;

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

pub trait AnyClone: Any + Clone {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> AnyClone for T where T: Any + Clone {}
