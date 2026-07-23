use crate::{
    calibration_kind::{
        CORRECT_MILLIS, CORRECT_NANO, CORRECT_PICO, CalibrationKind, CalibrationObject,
    },
    resolutions::{Resolution, ResolutionSearch},
    util::calc_res,
};

mod address_resolver;

#[derive(Debug)]
pub struct E192;

impl ResolutionSearch for E192 {
    fn find(ck: CalibrationKind, co: CalibrationObject, range_id: u32) -> Option<Resolution> {
        if co == CalibrationObject::Gain {
            return Some(Resolution::new(1.0 / 1024.0));
        }
        // offsets are stored in the fundamental unit of measurement, so the resolution
        // has to be in the same unit (e.g. CurrentAdc is in nA, the offsets are in A, so we multiply by CORRECT_NANO)
        match ck {
            CalibrationKind::CurrentAdc => match range_id {
                0 => Some(Resolution::new(calc_res(200.0, 16) * CORRECT_PICO)),
                1 => Some(Resolution::new(calc_res(2.0, 16) * CORRECT_NANO)),
                2 => Some(Resolution::new(calc_res(20.0, 16) * CORRECT_NANO)),
                3 => Some(Resolution::new(calc_res(200.0, 16) * CORRECT_NANO)),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => Some(Resolution::new(calc_res(512.0, 10) * CORRECT_MILLIS)),
                _ => None,
            },
            _ => None,
        }
    }
}
