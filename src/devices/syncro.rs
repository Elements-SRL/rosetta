use crate::{
    calibration_kind::{
        CORRECT_MILLIS, CORRECT_NANO, CORRECT_PICO, CalibrationKind, CalibrationObject,
    },
    resolutions::{Resolution, ResolutionSearch},
    util::calc_res,
};

mod address_resolver;

#[derive(Debug)]
pub struct SyncroV1;

impl ResolutionSearch for SyncroV1 {
    fn find(ck: CalibrationKind, co: CalibrationObject, range_id: u32) -> Option<Resolution> {
        if co == CalibrationObject::Gain {
            return Some(Resolution::new(1.0 / 1024.0));
        }
        // offsets are stored in the foundamental unit of measurement, so the resoulution
        // have to be in the same unit (e.g. CurrentAdc is in nA, the offsets are in A, so we multipy by CORRECT_NANO)
        match ck {
            CalibrationKind::CurrentAdc => match range_id {
                0 => Some(Resolution::new(calc_res(10.0, 16) * CORRECT_NANO)),
                1 => Some(Resolution::new(calc_res(40.0, 16) * CORRECT_NANO)),
                2 => Some(Resolution::new(calc_res(40.0, 16) * CORRECT_NANO)),
                3 => Some(Resolution::new(calc_res(400.0, 16) * CORRECT_NANO)),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => Some(Resolution::new(0.00762939453125 * CORRECT_MILLIS)),
                _ => None,
            },
            CalibrationKind::ShuntResistance => match range_id {
                // 10e-6 would be CORRECT_NANO / CORRECT_MILLIS
                0 => Some(Resolution::new(
                    (calc_res(10.0, 16) / 0.125 / 16384.0) * 1e-6,
                )),
                1 => Some(Resolution::new(
                    (calc_res(40.0, 16) / 0.125 / 16384.0) * 1e-6,
                )),
                2 => Some(Resolution::new(
                    (calc_res(40.0, 16) / 0.125 / 16384.0) * 1e-6,
                )),
                3 => Some(Resolution::new(
                    (calc_res(400.0, 16) / 0.125 / 16384.0) * 1e-6,
                )),
                _ => None,
            },
            CalibrationKind::CurrentDac => match range_id {
                0 => Some(Resolution::new(1.953125 * CORRECT_PICO)),
                1 => Some(Resolution::new(0.48828125 * CORRECT_PICO)),
                _ => None,
            },
            CalibrationKind::VoltageDac | CalibrationKind::RsCorrection => {
                Some(Resolution::new(0.125 * CORRECT_MILLIS))
            }
        }
    }
}

#[cfg(test)]
mod syncro_resolution_tests {

    use crate::util::calc_res;

    #[test]
    fn current_adc_resolution_test() {
        assert_eq!(calc_res(10.0, 16), 0.00030517578125);
        assert_eq!(calc_res(40.0, 16), 0.001220703125);
        assert_eq!(calc_res(40.0, 16), 0.001220703125);
        assert_eq!(calc_res(400.0, 16), 0.01220703125);
    }
}
