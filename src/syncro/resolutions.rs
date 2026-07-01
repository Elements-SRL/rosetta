pub struct Resoulutions {
    gain: f64,
    offset: f64,
}

impl Resoulutions {
    pub fn new(gain: f64, offset: f64) -> Self {
        Self { gain, offset }
    }
    pub fn scale_gain(&self, gain: f64) -> u16 {
        (gain / self.gain).round() as u16
    }
    pub fn scale_offset(&self, offset: f64) -> u16 {
        (offset / self.offset).round() as u16
    }
}
