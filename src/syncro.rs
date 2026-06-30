use crate::{
    models::{Calibration, read_calibtations},
    syncro::dev_ram::Ram,
};
use std::path::Path;

pub mod dev_ram;

struct Resoulutions {
    gain: f32,
    offset: f32,
}

impl Resoulutions {
    pub fn new(gain: f32, offset: f32) -> Self {
        Self { gain, offset }
    }
    pub fn scale_gain(&self, gain: f32) -> u16 {
        (gain / self.gain).round() as u16
    }
    pub fn scale_offset(&self, offset: f32) -> u16 {
        (offset / self.offset).round() as u16
    }
}

enum CalibrationKind {
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
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
            CalibrationKind::VoltageAdc => match range_id {
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
            CalibrationKind::ShuntResistance => match range_id {
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
            CalibrationKind::RsCorrection => match range_id {
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
            CalibrationKind::CurrentDac => match range_id {
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
            CalibrationKind::VoltageDac => match range_id {
                0 => Some(Resoulutions::new(1.0, 1.0)),
                _ => None,
            },
        }
    }
}

pub struct SyncroV1(Calibration);

impl SyncroV1 {
    pub fn from_file<I: AsRef<Path>>(path: I) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(read_calibtations(path)?))
    }

    pub fn calib_to_ram(&self, board_number: u32) -> Option<Ram> {
        let boards = &self.0.boards;
        todo!()
    }
}
