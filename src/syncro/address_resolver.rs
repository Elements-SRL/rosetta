use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject},
    models::{Board, RangeBlock, RangeSamplingRate, SamplingRate},
};

fn sr_id_to_clock_div(sr_id: u32) -> u16 {
    if sr_id < 3 { 2 } else { 1 }
}

fn calibration_object_to_bit(co: CalibrationObject) -> u16 {
    if co == CalibrationObject::Gain { 0 } else { 1 }
}

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

fn get_bit_7_6_5(ck: &CalibrationKind, sr_id: u32, range_id: u32, co: CalibrationObject) -> u16 {
    let cb = calibration_object_to_bit(co);
    let r_u16 = range_id as u16;
    let p = match ck {
        CalibrationKind::CurrentAdc | CalibrationKind::VoltageAdc => {
            sr_id_to_clock_div(sr_id) << 6 | cb << 5
        }
        CalibrationKind::VoltageDac => cb << 5,
        CalibrationKind::CurrentDac => 0x80 | r_u16 << 6 | cb << 5,
        CalibrationKind::ShuntResistance => r_u16 << 5,
        CalibrationKind::RsCorrection => 0x80 | r_u16 << 5,
    };
    p & 0xE0
}

// pub fn resolve(ck: CalibrationKind, rb: RangeBlock) -> u16 {
// }

#[cfg(test)]
mod address_resolver_test {
    use crate::{
        calibration_kind::{CalibrationKind, CalibrationObject},
        syncro::address_resolver::{get_bit_7_6_5, get_bit_10},
    };

    fn bit_7_6_5_helper(
        sr_range: core::ops::Range<u32>,
        adc_range: core::ops::Range<u32>,
        ck: CalibrationKind,
        co: CalibrationObject,
        res: u16,
    ) {
        sr_range.for_each(|sr| {
            adc_range.clone().for_each(|adc_range| {
                // slow
                assert_eq!(get_bit_7_6_5(&ck, sr, adc_range, co), res);
            })
        });
    }

    #[test]
    fn test_get_bit_10() {
        assert_eq!(get_bit_10(CalibrationKind::CurrentAdc), 0);
        vec![
            CalibrationKind::VoltageAdc,
            CalibrationKind::ShuntResistance,
            CalibrationKind::RsCorrection,
            CalibrationKind::CurrentDac,
            CalibrationKind::VoltageDac,
        ]
        .into_iter()
        .for_each(|ck| assert_eq!(get_bit_10(ck), 0x400));
    }

    #[test]
    fn test_get_bit_7_6_5_current_adc() {
        bit_7_6_5_helper(
            0..3,
            0..4,
            CalibrationKind::CurrentAdc,
            CalibrationObject::Gain,
            0x80,
        );
        bit_7_6_5_helper(
            0..3,
            0..4,
            CalibrationKind::CurrentAdc,
            CalibrationObject::Offset,
            0xA0,
        );
        bit_7_6_5_helper(
            3..5,
            0..4,
            CalibrationKind::CurrentAdc,
            CalibrationObject::Gain,
            0x40,
        );
        bit_7_6_5_helper(
            3..5,
            0..4,
            CalibrationKind::CurrentAdc,
            CalibrationObject::Offset,
            0x60,
        );
    }

    #[test]
    fn test_get_bit_7_6_5_voltage_adc() {
        bit_7_6_5_helper(
            0..3,
            0..4,
            CalibrationKind::VoltageAdc,
            CalibrationObject::Gain,
            0x80,
        );
        bit_7_6_5_helper(
            0..3,
            0..4,
            CalibrationKind::VoltageAdc,
            CalibrationObject::Offset,
            0xA0,
        );
        bit_7_6_5_helper(
            3..5,
            0..4,
            CalibrationKind::VoltageAdc,
            CalibrationObject::Gain,
            0x40,
        );
        bit_7_6_5_helper(
            3..5,
            0..4,
            CalibrationKind::VoltageAdc,
            CalibrationObject::Offset,
            0x60,
        );
    }

    #[test]
    fn test_get_bit_7_6_5_voltage_dac() {
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::VoltageDac,
            CalibrationObject::Gain,
            0,
        );
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::VoltageDac,
            CalibrationObject::Offset,
            0x20,
        );
    }

    #[test]
    fn test_get_bit_7_6_5_current_dac() {
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::CurrentDac,
            CalibrationObject::Gain,
            0x80,
        );
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::CurrentDac,
            CalibrationObject::Offset,
            0xA0,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::CurrentDac,
            CalibrationObject::Gain,
            0xC0,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::CurrentDac,
            CalibrationObject::Offset,
            0xE0,
        );
    }

    #[test]
    fn test_get_bit_7_6_5_shunt_resistance() {
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Gain,
            0,
        );
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Offset,
            0,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Gain,
            0x20,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Offset,
            0x20,
        );
        bit_7_6_5_helper(
            0..5,
            2..3,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Gain,
            0x40,
        );
        bit_7_6_5_helper(
            0..5,
            2..3,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Offset,
            0x40,
        );
        bit_7_6_5_helper(
            0..5,
            3..4,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Gain,
            0x60,
        );
        bit_7_6_5_helper(
            0..5,
            3..4,
            CalibrationKind::ShuntResistance,
            CalibrationObject::Offset,
            0x60,
        );
    }

    #[test]
    fn test_get_bit_7_6_5_rs_correction() {
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::RsCorrection,
            CalibrationObject::Gain,
            0x80,
        );
        bit_7_6_5_helper(
            0..5,
            0..1,
            CalibrationKind::RsCorrection,
            CalibrationObject::Offset,
            0x80,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::RsCorrection,
            CalibrationObject::Gain,
            0xA0,
        );
        bit_7_6_5_helper(
            0..5,
            1..2,
            CalibrationKind::RsCorrection,
            CalibrationObject::Offset,
            0xA0,
        );
        bit_7_6_5_helper(
            0..5,
            2..3,
            CalibrationKind::RsCorrection,
            CalibrationObject::Gain,
            0xC0,
        );
        bit_7_6_5_helper(
            0..5,
            2..3,
            CalibrationKind::RsCorrection,
            CalibrationObject::Offset,
            0xC0,
        );
        bit_7_6_5_helper(
            0..5,
            3..4,
            CalibrationKind::RsCorrection,
            CalibrationObject::Gain,
            0xE0,
        );
        bit_7_6_5_helper(
            0..5,
            3..4,
            CalibrationKind::RsCorrection,
            CalibrationObject::Offset,
            0xE0,
        );
    }
}
