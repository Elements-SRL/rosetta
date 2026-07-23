use crate::{
    address_resolver::AddressResolver,
    calibration_kind::{CalibrationKind, CalibrationObject},
    models::{Board, Calibration, RangeBlock, read_calibtations},
    resolutions::ResolutionSearch,
    util::divide,
};
use e384_rust::device::Device;
use std::{fmt::Debug, marker::PhantomData, path::Path, thread, time::Duration};
use tracing::instrument;

#[derive(Debug)]
pub struct Stele<D: AddressResolver + ResolutionSearch + Debug> {
    calibration: Calibration,
    dev: Device,
    d: PhantomData<D>,
}

impl<D: AddressResolver + ResolutionSearch + Debug> Stele<D> {
    pub fn new<I: AsRef<Path>>(path: I, dev: Device) -> Result<Self, Box<dyn std::error::Error>> {
        let calibration = read_calibtations(path)?;
        // let dev = Device::connect(device_id)
        //     .map_err(|e| format!("failed to connect to device (error code {e:?})"))?;
        Ok(Self {
            dev,
            calibration,
            d: PhantomData,
        })
    }

    #[instrument(level = "trace")]
    fn set_calibrations(
        &self,
        calibs: &[f64],
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
        clk_div: Option<u16>,
    ) -> Option<()> {
        match D::find(ck, co, range_id) {
            Some(res) => {
                calibs.iter().enumerate().for_each(|(ch_idx, g)| {
                    let v = res.scale(*g);
                    let (msb, lsb) = divide(v);
                    let ch_idx = ch_idx as u16;
                    let (add_lsb, add_msb) = D::resolve(ck, range_id, sr_id, co, ch_idx, clk_div);
                    if let Err(e) = self.dev.ok_write_calibration_ram(add_lsb.0, lsb.0) {
                        tracing::error!("failed to write calibration ram: {e:?}");
                    }
                    if let Err(e) = self.dev.ok_write_calibration_ram(add_msb.0, msb.0) {
                        tracing::error!("failed to write calibration ram: {e:?}");
                    }
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
    fn apply_calib_step(&self, rbs: Vec<RangeBlock>, ck: CalibrationKind) {
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
                    sr.clk_div,
                );
                self.set_calibrations(
                    &sr.calibrations.offsets,
                    ck,
                    range_id,
                    sr_id,
                    CalibrationObject::Offset,
                    sr.clk_div,
                );
            });
        });
    }

    #[instrument(level = "trace")]
    fn apply_board(&self, board: Board) {
        tracing::info!("Board: {}", board.board_number);
        let bn = board.board_number as u16;
        if let Err(e) = self.dev.ok_select_calibration_ram(bn) {
            tracing::error!("failed to select calibration ram: {e:?}");
        }
        self.apply_calib_step(board.current_adc, CalibrationKind::CurrentAdc);
        self.apply_calib_step(board.current_dac, CalibrationKind::CurrentDac);
        self.apply_calib_step(board.voltage_adc, CalibrationKind::VoltageAdc);
        self.apply_calib_step(board.voltage_dac, CalibrationKind::VoltageDac);
        self.apply_calib_step(board.shunt_resistance, CalibrationKind::ShuntResistance);
        self.apply_calib_step(board.rs_correction, CalibrationKind::RsCorrection);
    }

    pub fn apply_complete_calibration(&mut self) {
        if let Err(e) = self.dev.set_debug_bit(0, 14, false, true) {
            tracing::error!("failed to set debug bit: {e:?}");
        }
        tracing::info!("Waiting 10s...");
        thread::sleep(Duration::from_secs(10));
        tracing::info!("Done!");
        if let Err(e) = self.dev.ok_move_calibration_eeprom_to_rams() {
            tracing::error!("failed to move calibration eeprom to rams: {e:?}");
        }
        self.calibration.boards.clone().into_iter().for_each(|b| {
            // todo rmeove this if statemente for real deal
            if b.board_number < 6 {
                self.apply_board(b)
            }
        });
        tracing::info!("Calibration completed!");
        if let Err(e) = self.dev.ok_move_calibration_rams_to_eeprom() {
            tracing::error!("failed to move calibration rams to eeprom: {e:?}");
        }
        thread::sleep(Duration::from_secs(5));
        tracing::info!("See you next time!");
    }
}
