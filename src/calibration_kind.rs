use crate::syncro::resolutions::Resoulution;

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
        match self {
            CalibrationKind::CurrentAdc => match range_id {
                0 => Some(Resoulution::new(0.00030517578125)),
                1 => Some(Resoulution::new(0.001220703125)),
                2 => Some(Resoulution::new(0.001220703125)),
                3 => Some(Resoulution::new(0.01220703125)),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => Some(Resoulution::new(0.00762939453125)),
                _ => None,
            },
            CalibrationKind::ShuntResistance => match range_id {
                0 => Some(Resoulution::new(0.00030517578125 / 0.125 / 16384.0)),
                1 => Some(Resoulution::new(0.001220703125 / 0.125 / 16384.0)),
                2 => Some(Resoulution::new(0.001220703125 / 0.125 / 16384.0)),
                3 => Some(Resoulution::new(0.01220703125 / 0.125 / 16384.0)),
                _ => None,
            },
            CalibrationKind::CurrentDac => match range_id {
                0 => Some(Resoulution::new(1.953125)),
                1 => Some(Resoulution::new(0.48828125)),
                _ => None,
            },
            CalibrationKind::VoltageDac | CalibrationKind::RsCorrection => {
                Some(Resoulution::new(0.125))
            }
        }
    }
}
