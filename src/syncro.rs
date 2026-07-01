use crate::{
    models::{Board, Calibration, read_calibtations},
    syncro::dev_ram::Ram,
};
use std::path::Path;

pub mod address_resolver;
pub mod dev_ram;
pub mod resolutions;

const BOARDS: u32 = 24;

pub struct SyncroV1 {
    calibration: Calibration,
    rams: Vec<Ram>,
}

impl SyncroV1 {
    pub fn from_file<I: AsRef<Path>>(path: I) -> Result<Self, Box<dyn std::error::Error>> {
        let calibration = read_calibtations(path)?;
        let rams = (0..BOARDS).map(Ram::new).collect();
        Ok(Self { calibration, rams })
    }

    fn board_to_ram(board: &Board, r: Ram) -> Ram {
        // board.board_number
        todo!()
    }

    pub fn calib_to_ram(&self, board_number: u32) -> Option<Ram> {
        let boards = &self.calibration.boards;
        todo!()
    }
}
