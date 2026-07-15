use crate::syncro::resolutions::Resoulution;

const CORRECT_NANO: f64 = 1e-9;
const CORRECT_PICO: f64 = 1e-12;
const CORRECT_MILLIS: f64 = 1e-3;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CalibrationObject {
    Gain,
    Offset,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CalibrationKind {
    CurrentAdc,
    VoltageAdc,
    ShuntResistance,
    RsCorrection,
    CurrentDac,
    VoltageDac,
}

impl CalibrationKind {
    pub fn resolution(&self, range_id: u32, co: CalibrationObject) -> Option<Resoulution> {
        if co == CalibrationObject::Gain {
            return Some(Resoulution::new(1.0 / 1024.0));
        }
        // offsets are stored in the foundamental unit of measurement, so the resoulution 
        // have to be in the same unit (e.g. CurrentAdc is in nA, the offsets are in A, so we multipy by CORRECT_NANO)
        match self {
            CalibrationKind::CurrentAdc => match range_id {
                0 => Some(Resoulution::new(0.00030517578125 * CORRECT_NANO)),
                1 => Some(Resoulution::new(0.001220703125 * CORRECT_NANO)),
                2 => Some(Resoulution::new(0.001220703125 * CORRECT_NANO)),
                3 => Some(Resoulution::new(0.01220703125 * CORRECT_NANO)),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => Some(Resoulution::new(0.00762939453125 * CORRECT_MILLIS)),
                _ => None,
            },
            CalibrationKind::ShuntResistance => match range_id {
                // 10e-6 would be CORRECT_NANO / CORRECT_MILLIS
                0 => Some(Resoulution::new((0.00030517578125 / 0.125 / 16384.0) * 1e-6)),
                1 => Some(Resoulution::new((0.001220703125 / 0.125/ 16384.0) * 1e-6)),
                2 => Some(Resoulution::new((0.001220703125 / 0.125 / 16384.0) * 1e-6)),
                3 => Some(Resoulution::new((0.01220703125 / 0.125 / 16384.0) * 1e-6)),
                _ => None,
            },
            CalibrationKind::CurrentDac => match range_id {
                0 => Some(Resoulution::new(1.953125 * CORRECT_PICO)),
                1 => Some(Resoulution::new(0.48828125 * CORRECT_PICO)),
                _ => None,
            },
            CalibrationKind::VoltageDac | CalibrationKind::RsCorrection => {
                Some(Resoulution::new(0.125 * CORRECT_MILLIS))
            }
        }
    }
}
