use crate::{
    address_resolver::AddressResolver,
    calibration_kind::{CalibrationKind, CalibrationObject},
    syncro::SyncroV1,
    util::{Lsb, Msb},
};

fn sr_id_to_clock_div(sr_id: u16) -> u16 {
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

fn get_bit_9_8(ck: CalibrationKind, range_id: u16) -> u16 {
    let p = match ck {
        CalibrationKind::CurrentAdc => range_id << 8,
        CalibrationKind::VoltageAdc => 0,
        CalibrationKind::VoltageDac | CalibrationKind::CurrentDac => 0x100,
        CalibrationKind::ShuntResistance | CalibrationKind::RsCorrection => 0x200,
    };
    p & 0x300
}

fn get_bit_7_6_5(ck: CalibrationKind, sr_id: u16, range_id: u16, co: CalibrationObject) -> u16 {
    let cb = calibration_object_to_bit(co);
    let p = match ck {
        CalibrationKind::CurrentAdc | CalibrationKind::VoltageAdc => {
            sr_id_to_clock_div(sr_id) << 6 | cb << 5
        }
        CalibrationKind::VoltageDac => cb << 5,
        CalibrationKind::CurrentDac => 0x80 | range_id << 6 | cb << 5,
        CalibrationKind::ShuntResistance => range_id << 5,
        CalibrationKind::RsCorrection => 0x80 | range_id << 5,
    };
    p & 0xE0
}

fn get_bit_4_3_2_1(ch_idx: u16) -> u16 {
    ch_idx << 1 & 0x1E
}

impl AddressResolver for SyncroV1 {
    type Address = (Lsb<u16>, Msb<u16>);

    fn resolve(
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
        ch_idx: u16,
    ) -> Self::Address {
        let range_id = range_id as u16;
        let sr_id = sr_id as u16;
        let b10 = get_bit_10(ck);
        let b9_8 = get_bit_9_8(ck, range_id);
        let b_7_6_5 = get_bit_7_6_5(ck, sr_id, range_id, co);
        let b4_3_2_1 = get_bit_4_3_2_1(ch_idx);
        let address = b10 | b9_8 | b_7_6_5 | b4_3_2_1;
        (Lsb(address | 1), Msb(address))
    }
}

#[cfg(test)]
mod address_resolver_test {
    use crate::{
        calibration_kind::{CalibrationKind, CalibrationObject},
        syncro::address_resolver::{get_bit_4_3_2_1, get_bit_7_6_5, get_bit_9_8, get_bit_10},
    };
    const R0: core::ops::Range<u16> = 0..1;
    const R1: core::ops::Range<u16> = 1..2;
    const R2: core::ops::Range<u16> = 2..3;
    const R3: core::ops::Range<u16> = 3..4;

    const R0_3: core::ops::Range<u16> = 0..3;
    const R0_4: core::ops::Range<u16> = 0..4;
    const R0_5: core::ops::Range<u16> = 0..5;
    const R3_5: core::ops::Range<u16> = 3..5;

    fn bit_7_6_5_helper(
        sr_range: core::ops::Range<u16>,
        adc_range: core::ops::Range<u16>,
        ck: CalibrationKind,
        co: CalibrationObject,
        res: u16,
    ) {
        sr_range.for_each(|sr| {
            adc_range.clone().for_each(|adc_range| {
                // slow
                assert_eq!(get_bit_7_6_5(ck, sr, adc_range, co), res);
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
    fn test_get_bit_9_8() {
        vec![(0, 0), (1, 0x100), (2, 0x200), (3, 0x300)]
            .into_iter()
            .for_each(|(range_id, res)| {
                assert_eq!(get_bit_9_8(CalibrationKind::CurrentAdc, range_id), res)
            });
        assert_eq!(get_bit_9_8(CalibrationKind::VoltageAdc, 0), 0);
        assert_eq!(get_bit_9_8(CalibrationKind::VoltageDac, 0), 0x100);
        assert_eq!(get_bit_9_8(CalibrationKind::CurrentDac, 0), 0x100);
        assert_eq!(get_bit_9_8(CalibrationKind::ShuntResistance, 0), 0x200);
        assert_eq!(get_bit_9_8(CalibrationKind::RsCorrection, 0), 0x200);
    }

    #[test]
    fn test_get_bit_7_6_5_current_adc() {
        let ca = CalibrationKind::CurrentAdc;
        let g = CalibrationObject::Gain;
        let o = CalibrationObject::Offset;
        bit_7_6_5_helper(R0_3, R0_4, ca, g, 0x80);
        bit_7_6_5_helper(R0_3, R0_4, ca, o, 0xA0);
        bit_7_6_5_helper(R3_5, R0_4, ca, g, 0x40);
        bit_7_6_5_helper(R3_5, R0_4, ca, o, 0x60);
    }

    #[test]
    fn test_get_bit_7_6_5_voltage_adc() {
        let va = CalibrationKind::VoltageAdc;
        let g = CalibrationObject::Gain;
        let o = CalibrationObject::Offset;
        bit_7_6_5_helper(R0_3, R0_4, va, g, 0x80);
        bit_7_6_5_helper(R0_3, R0_4, va, o, 0xA0);
        bit_7_6_5_helper(R3_5, R0_4, va, g, 0x40);
        bit_7_6_5_helper(R3_5, R0_4, va, o, 0x60);
    }

    #[test]
    fn test_get_bit_7_6_5_voltage_dac() {
        let vd = CalibrationKind::VoltageDac;
        bit_7_6_5_helper(R0_5, R0, vd, CalibrationObject::Gain, 0);
        bit_7_6_5_helper(R0_5, R0, vd, CalibrationObject::Offset, 0x20);
    }

    #[test]
    fn test_get_bit_7_6_5_current_dac() {
        let cd: CalibrationKind = CalibrationKind::CurrentDac;
        bit_7_6_5_helper(R0_5, R0, cd, CalibrationObject::Gain, 0x80);
        bit_7_6_5_helper(R0_5, R0, cd, CalibrationObject::Offset, 0xA0);
        bit_7_6_5_helper(R0_5, R1, cd, CalibrationObject::Gain, 0xC0);
        bit_7_6_5_helper(R0_5, R1, cd, CalibrationObject::Offset, 0xE0);
    }

    #[test]
    fn test_get_bit_7_6_5_shunt_resistance() {
        let sr = CalibrationKind::ShuntResistance;
        bit_7_6_5_helper(R0_5, R0, sr, CalibrationObject::Gain, 0);
        bit_7_6_5_helper(R0_5, R0, sr, CalibrationObject::Offset, 0);
        bit_7_6_5_helper(R0_5, R1, sr, CalibrationObject::Gain, 0x20);
        bit_7_6_5_helper(R0_5, R1, sr, CalibrationObject::Offset, 0x20);
        bit_7_6_5_helper(R0_5, R2, sr, CalibrationObject::Gain, 0x40);
        bit_7_6_5_helper(R0_5, R2, sr, CalibrationObject::Offset, 0x40);
        bit_7_6_5_helper(R0_5, R3, sr, CalibrationObject::Gain, 0x60);
        bit_7_6_5_helper(R0_5, R3, sr, CalibrationObject::Offset, 0x60);
    }

    #[test]
    fn test_get_bit_7_6_5_rs_correction() {
        let rsc = CalibrationKind::RsCorrection;
        bit_7_6_5_helper(R0_5, R0, rsc, CalibrationObject::Gain, 0x80);
        bit_7_6_5_helper(R0_5, R0, rsc, CalibrationObject::Offset, 0x80);
        bit_7_6_5_helper(R0_5, R1, rsc, CalibrationObject::Gain, 0xA0);
        bit_7_6_5_helper(R0_5, R1, rsc, CalibrationObject::Offset, 0xA0);
        bit_7_6_5_helper(R0_5, R2, rsc, CalibrationObject::Gain, 0xC0);
        bit_7_6_5_helper(R0_5, R2, rsc, CalibrationObject::Offset, 0xC0);
        bit_7_6_5_helper(R0_5, R3, rsc, CalibrationObject::Gain, 0xE0);
        bit_7_6_5_helper(R0_5, R3, rsc, CalibrationObject::Offset, 0xE0);
    }

    #[test]
    fn test_get_bit_4_3_2_1() {
        (0..16).for_each(|ch| assert_eq!(get_bit_4_3_2_1(ch), ch * 2))
    }
}
