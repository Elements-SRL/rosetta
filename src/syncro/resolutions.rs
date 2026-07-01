pub struct Resoulutions {
    gain: f32,
    offset: f32,
}

impl Resoulutions {
    pub fn new(gain: f32, offset: f32) -> Self {
        Self { gain, offset }
    }
    pub fn scale_gain(&self, gain: f32) -> u16 {
        (gain / self.gain).round() as u16
    }
    pub fn scale_offset(&self, offset: f32) -> u16 {
        (offset / self.offset).round() as u16
    }
}
