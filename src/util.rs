#[derive(Debug, PartialEq)]
pub struct Lsb<T>(pub T);
#[derive(Debug, PartialEq)]
pub struct Msb<T>(pub T);

pub fn divide(v: i16) -> (Msb<u8>, Lsb<u8>) {
    (Msb((v >> 8) as u8), Lsb(v as u8))
}

pub fn calc_res(value_to: f64, bits: i32) -> f64 {
    value_to / 2.0_f64.powi(bits - 1)
}

/// Formats `t` as a file-name-safe UTC timestamp: `2026-11-24_153012UTC`.
pub fn utc_stamp(t: std::time::SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Days since epoch -> civil date (Howard Hinnant's algorithm, days >= 0).
    let z = days + 719_468;
    let (era, doe) = (z / 146_097, z % 146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = era * 400 + yoe + u64::from(month <= 2);
    let (h, m, s) = (rem / 3600, rem % 3600 / 60, rem % 60);
    format!("{year:04}-{month:02}-{day:02}_{h:02}{m:02}{s:02}UTC")
}

#[cfg(test)]
mod util_tests {

    use crate::util::{Lsb, Msb, divide, utc_stamp};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn utc_stamp_formats_known_instants() {
        assert_eq!(utc_stamp(UNIX_EPOCH), "1970-01-01_000000UTC");
        // Leap day.
        let t = UNIX_EPOCH + Duration::from_secs(1_709_210_096);
        assert_eq!(utc_stamp(t), "2024-02-29_123456UTC");
        // Century non-leap boundary: 2100-03-01 00:00:00.
        let t = UNIX_EPOCH + Duration::from_secs(4_107_542_400);
        assert_eq!(utc_stamp(t), "2100-03-01_000000UTC");
    }

    #[test]
    fn divide_test() {
        let (msb, lsb) = divide(0x0F);
        assert_eq!(msb, Msb(0));
        assert_eq!(lsb, Lsb(0xF));
    }
}
