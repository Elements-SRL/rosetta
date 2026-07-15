pub struct Resolution(f64);

impl Resolution {
    pub fn new(res: f64) -> Self {
        Self(res)
    }
    pub fn scale(&self, other: f64) -> u16 {
        (other / self.0).round() as u16
    }
}
