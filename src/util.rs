#[derive(Debug, PartialEq)]
pub struct Lsb<T>(pub T);
#[derive(Debug, PartialEq)]
pub struct Msb<T>(pub T);

pub fn divide(v: i16) -> (Msb<u8>, Lsb<u8>) {
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
