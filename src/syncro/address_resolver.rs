use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject},
    models::{Board, RangeBlock, RangeSamplingRate, SamplingRate},
};

////////////////////////////////////
// VALUES HAVE BEEN GUESSED
fn sr_id_to_clock_div(sr_id: u32) -> u16 {
    if sr_id < 3 { 0 } else { 1 }
}
////////////////////////////////////

////////////////////////////////////
// VALUES HAVE BEEN GUESSED
fn calibration_object_to_bit(co: CalibrationObject) -> u16 {
    if co == CalibrationObject::Gain { 1 } else { 0 }
}
////////////////////////////////////

fn get_bit_10(ck: CalibrationKind) -> u16 {
    if ck == CalibrationKind::CurrentAdc {
        0
    } else {
        0x400
    }
}

fn get_bit_9_8(ck: &CalibrationKind, rb: RangeBlock) -> u16 {
    let p = match ck {
        CalibrationKind::CurrentAdc => (rb.range_id as u16) << 8,
        CalibrationKind::VoltageAdc => 0,
        CalibrationKind::VoltageDac | CalibrationKind::CurrentDac => 0x500,
        CalibrationKind::ShuntResistance | CalibrationKind::RsCorrection => 0xC00,
    };
    p & 0x300
}

fn get_bit_7_6_5(ck: &CalibrationKind, sr_id: u32, adc_range: u32, co: CalibrationObject) -> u16 {
    let p = match ck {
        CalibrationKind::CurrentAdc | CalibrationKind::VoltageAdc => {
            sr_id_to_clock_div(sr_id) << 6 | calibration_object_to_bit(co) << 5
        }
        CalibrationKind::VoltageDac => calibration_object_to_bit(co) << 5,
        CalibrationKind::CurrentDac => todo!(),
        CalibrationKind::ShuntResistance => todo!(),
        CalibrationKind::RsCorrection => sr_id_to_clock_div(sr_id) << 5,
    };
    p & 0xE0
}

// pub fn resolve(ck: CalibrationKind, rb: RangeBlock) -> u16 {
// }
