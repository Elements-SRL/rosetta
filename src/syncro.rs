use tracing::instrument;
use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject}, e384_commands::{get_ram, write_u8}, models::{Board, Calibration, RangeBlock, read_calibtations}, syncro::{address_resolver::resolve, dev_ram::Ram}, util::divide,
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


    #[instrument]
    fn apply_calib_step(rbs: Vec<RangeBlock>, ck: CalibrationKind) {
        tracing::info!("CalibrationKind: {:?}", ck);
        rbs.iter()
        .for_each(|rb| {
            tracing::info!("range: {}, idx: {}", rb.range_name, rb.range_id);
            let range_id = rb.range_id;
            rb.sampling_rates.iter()
            .for_each(|sr| {
                let sr_id = sr.sr_id;
                tracing::info!("sr id: {}", sr_id);
                match ck.resolutios(range_id) {
                    Some(res) => {
                        sr.calibrations.gains.iter()
                        .enumerate()
                        .for_each(|(ch_idx, g)| {
                            let v = res.scale_gain(*g);
                            let (msb, lsb) = divide(v);
                            let ch_idx = ch_idx as u16;
                            let (add_lsb, add_msb) = resolve(ck, range_id, sr_id, CalibrationObject::Gain, ch_idx);
                            write_u8(add_lsb.0, lsb.0);
                            write_u8(add_msb.0, msb.0);
                        });
                        sr.calibrations.offsets.iter()
                        .enumerate()
                        .for_each(|(ch_idx, o)| {
                            let v = res.scale_offset(*o);
                            let (msb, lsb) = divide(v);
                            let ch_idx = ch_idx as u16;
                            let (add_lsb, add_msb) = resolve(ck, range_id, sr_id, CalibrationObject::Offset, ch_idx);
                            write_u8(add_lsb.0, lsb.0);
                            write_u8(add_msb.0, msb.0);
                        });
                    }
                    None => {
                        tracing::error!("No resolution for this set of values");
                    }
                }
            });
        });
        todo!()
    }

    fn apply_board(board: &Board) -> Ram {
        let bn = board.board_number;

        let ram_content: [u8; 2048] = get_ram(bn);
        todo!()
    }

    pub fn calib_to_ram(&self, board_number: u32) -> Option<Ram> {
        let boards = &self.calibration.boards;
        todo!()
    }
}
