pub struct Resolution(f64);

impl Resolution {
    pub fn new(res: f64) -> Self {
        Self(res)
    }
    //Might be bettere to treat gains as u16
    pub fn scale(&self, other: f64) -> i16 {
        (other / self.0).round() as i16
    }
}

#[cfg(test)]
mod resolution_test {
    use crate::{
        calibration_kind::CORRECT_NANO,
        resolutions::Resolution,
        util::{Lsb, Msb, divide},
    };

    #[test]
    fn offset_resolution_positive() {
        let r = Resolution::new(0.00030517578125 * CORRECT_NANO);
        let (msb, lsb) = divide(r.scale(2e-9));
        assert_eq!(msb, Msb(25));
        assert_eq!(lsb, Lsb(154));
    }

    #[test]
    fn offset_resolution_negative() {
        let r = Resolution::new(0.00030517578125 * CORRECT_NANO);
        let (msb, lsb) = divide(r.scale(-2e-9));
        assert_eq!(msb, Msb(230));
        assert_eq!(lsb, Lsb(102));
    }

    #[test]
    fn gain_resolution_positive() {
        let r = Resolution::new(1.0 / 1024.0);
        let g = r.scale(1.05);
        assert_eq!(1075, g);
    }
}
