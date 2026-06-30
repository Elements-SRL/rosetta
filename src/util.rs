#[derive(Debug, PartialEq)]
pub struct Lsb(pub u8);
#[derive(Debug, PartialEq)]
pub struct Msb(pub u8);

pub fn divide(v: u16) -> (Msb, Lsb) {
    (Msb((v >> 8) as u8), Lsb(v as u8))
}

#[cfg(test)]
mod util_tests {

    use crate::util::{Lsb, Msb, divide};

    #[test]
    fn divide_test() {
        let (msb, lsb) = divide(0x0F);
        assert_eq!(msb, Msb(0));
        assert_eq!(lsb, Lsb(0xF));
    }
}
