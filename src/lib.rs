pub mod models;


pub fn poop() -> usize {
    42
}


#[cfg(test)]
mod lib_tests {
    use crate::poop;

    #[test]
    fn poop_test() {
        assert_eq!(poop(), 42)
    }
}