pub struct Resoulution(f64);

impl Resoulution {
    pub fn new(res: f64) -> Self {
        Self(res)
    }
    pub fn scale(&self, other: f64) -> u16 {
        (other / self.0).round() as u16
    }
}
