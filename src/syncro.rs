use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject}, e384_commands::E384MiniWrapper, models::{Board, Calibration, RangeBlock, read_calibtations}, syncro::address_resolver::resolve, util::divide,
};
use std::path::Path;
use tracing::instrument;

pub mod address_resolver;
pub mod resolutions;

#[derive(Debug)]
pub struct SyncroV1 {
    calibration: Calibration,
    dev: E384MiniWrapper,
}

impl SyncroV1 {
    pub fn from_file<I: AsRef<Path>>(
        path: I,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let calibration = read_calibtations(path)?;
        let dev = E384MiniWrapper::connect_to_first_device()
            .map_err(|e| format!("failed to connect to device (error code {e})"))?;

        Ok(Self { dev, calibration })
    }

    #[instrument(level = "trace")]
    fn set_calibrations(
        &mut self,
        calibs: &[f64],
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
    ) -> Option<()> {
        match ck.resolution(range_id, co) {
            Some(res) => {
                calibs.iter().enumerate().for_each(|(ch_idx, g)| {
                    let v = res.scale(*g);
                    let (msb, lsb) = divide(v);
                    let ch_idx = ch_idx as u16;
                    let (add_lsb, add_msb) = resolve(ck, range_id, sr_id, co, ch_idx);
                    self.dev.write_u8(add_lsb.0, lsb.0);
                    self.dev.write_u8(add_msb.0, msb.0);
                });
                Some(())
            }
            None => {
                tracing::error!("No resolution for this set of values");
                None
            }
        }
    }

    #[instrument(level = "trace")]
    fn apply_calib_step(&mut self, rbs: Vec<RangeBlock>, ck: CalibrationKind) {
        tracing::info!("CalibrationKind: {:?}", ck);
        rbs.iter().for_each(|rb| {
            tracing::info!("range: {}, idx: {}", rb.range_name, rb.range_id);
            let range_id = rb.range_id;
            rb.sampling_rates.iter().for_each(|sr| {
                let sr_id = sr.sr_id;
                tracing::info!("sr id: {}", sr_id);
                self.set_calibrations(
                    &sr.calibrations.gains,
                    ck,
                    range_id,
                    sr_id,
                    CalibrationObject::Gain,
                );
                self.set_calibrations(
                    &sr.calibrations.offsets,
                    ck,
                    range_id,
                    sr_id,
                    CalibrationObject::Offset,
                );
            });
        });
    }

    #[instrument(level = "trace")]
    fn apply_board(&mut self, board: Board) {
        let bn = board.board_number as u16;
        // self.dev.get_ram(bn);
        self.dev.set_ram(bn);
        self.apply_calib_step(board.current_adc, CalibrationKind::CurrentAdc);
        self.apply_calib_step(board.current_dac, CalibrationKind::CurrentDac);
        self.apply_calib_step(board.voltage_adc, CalibrationKind::VoltageAdc);
        self.apply_calib_step(board.voltage_dac, CalibrationKind::VoltageDac);
        self.apply_calib_step(board.shunt_resistance, CalibrationKind::ShuntResistance);
        self.apply_calib_step(board.rs_correction, CalibrationKind::RsCorrection);
    }

    pub fn apply_complete_calibration(&mut self) {
        self.dev.read_eeprom();
        self.calibration.boards.clone().into_iter().for_each(|b| self.apply_board(b));
        self.dev.write_all_eeproms();
    }
}
