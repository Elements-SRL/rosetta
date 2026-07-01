use crate::syncro::resolutions::Resoulutions;

#[derive(Debug, PartialEq)]
pub enum CalibrationObject {
    Gain,
    Offset,
}

#[derive(Debug, PartialEq)]
pub enum CalibrationKind {
    CurrentAdc,
    VoltageAdc,
    ShuntResistance,
    RsCorrection,
    CurrentDac,
    VoltageDac,
}

impl CalibrationKind {
    pub fn resolutios(&self, range_id: u32) -> Option<Resoulutions> {
        match self {
            CalibrationKind::CurrentAdc => match range_id {
                0 => todo!(),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => todo!(),
                _ => None,
            },
            CalibrationKind::ShuntResistance => match range_id {
                0 => todo!(),
                _ => None,
            },
            CalibrationKind::RsCorrection => match range_id {
                0 => todo!(),
                _ => None,
            },
            CalibrationKind::CurrentDac => match range_id {
                0 => todo!(),
                _ => None,
            },
            CalibrationKind::VoltageDac => match range_id {
                0 => todo!(),
                _ => None,
            },
        }
    }
}
