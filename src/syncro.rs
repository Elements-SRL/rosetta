use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject}, e384_commands::{get_ram, read_eeprom, set_ram, write_all_eeproms, write_u8}, models::{Board, Calibration, RangeBlock, read_calibtations}, syncro::address_resolver::resolve, util::divide,
};
use std::path::Path;
use tracing::instrument;

pub mod address_resolver;
pub mod resolutions;

pub struct SyncroV1 {
    calibration: Calibration,
}

impl SyncroV1 {
    pub fn from_file<I: AsRef<Path>>(path: I) -> Result<Self, Box<dyn std::error::Error>> {
        let calibration = read_calibtations(path)?;
        Ok(Self { calibration })
    }

    #[instrument]
    fn apply_calib_step(rbs: Vec<RangeBlock>, ck: CalibrationKind) {
        tracing::info!("CalibrationKind: {:?}", ck);
        rbs.iter().for_each(|rb| {
            tracing::info!("range: {}, idx: {}", rb.range_name, rb.range_id);
            let range_id = rb.range_id;
            rb.sampling_rates.iter().for_each(|sr| {
                let sr_id = sr.sr_id;
                tracing::info!("sr id: {}", sr_id);
                match ck.resolution(range_id, CalibrationObject::Gain) {
                    Some(res) => {
                        sr.calibrations
                            .gains
                            .iter()
                            .enumerate()
                            .for_each(|(ch_idx, g)| {
                                let v = res.scale(*g);
                                let (msb, lsb) = divide(v);
                                let ch_idx = ch_idx as u16;
                                let (add_lsb, add_msb) =
                                    resolve(ck, range_id, sr_id, CalibrationObject::Gain, ch_idx);
                                write_u8(add_lsb.0, lsb.0);
                                write_u8(add_msb.0, msb.0);
                            });
                        sr.calibrations
                            .offsets
                            .iter()
                            .enumerate()
                            .for_each(|(ch_idx, o)| {
                                let v = res.scale(*o);
                                let (msb, lsb) = divide(v);
                                let ch_idx = ch_idx as u16;
                                let (add_lsb, add_msb) =
                                    resolve(ck, range_id, sr_id, CalibrationObject::Offset, ch_idx);
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
    }

    #[instrument]
    fn apply_board(board: Board) {
        let bn = board.board_number;
        get_ram(bn);
        set_ram(bn);
        SyncroV1::apply_calib_step(board.current_adc, CalibrationKind::CurrentAdc);
        SyncroV1::apply_calib_step(board.current_dac, CalibrationKind::CurrentDac);
        SyncroV1::apply_calib_step(board.voltage_adc, CalibrationKind::VoltageAdc);
        SyncroV1::apply_calib_step(board.voltage_dac, CalibrationKind::VoltageDac);
        SyncroV1::apply_calib_step(board.shunt_resistance, CalibrationKind::ShuntResistance);
        SyncroV1::apply_calib_step(board.rs_correction, CalibrationKind::RsCorrection);
    }

    pub fn apply_complete_calibration(self) {
        read_eeprom();
        self.calibration.boards.into_iter().for_each(SyncroV1::apply_board);
        write_all_eeproms();
    }
}
