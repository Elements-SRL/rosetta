pub struct Resolution(f64);

impl Resolution {
    pub fn new(res: f64) -> Self {
        Self(res)
    }
    pub fn scale(&self, other: f64) -> u16 {
        (other / self.0).round() as u16
    }
}


#[cfg(test)]
mod resolution_test {
    use crate::models::read_calibtations;

    #[test]
    fn offset_resolution() {
        
        // println!("{:#?}", calib);
        // Ok(())
    }
}
