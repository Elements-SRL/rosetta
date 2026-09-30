use crate::{
    address_resolver::AddressResolver,
    calibration_kind::{CalibrationKind, CalibrationObject},
    devices::e192::{E192, El08b},
    util::{Lsb, Msb},
};

fn bit_9(ck: CalibrationKind) -> u16 {
    let r = match ck {
        CalibrationKind::CurrentAdc => 0,
        _ => 1,
    };
    r << 9
}

fn bit_8_7(ck_div: u16, ck: CalibrationKind) -> u16 {
    let r = match ck {
        CalibrationKind::CurrentAdc => ck_div & 0b11,
        CalibrationKind::ShuntResistance => 1,
        CalibrationKind::VoltageDac => 0,
        _ => 0,
    };
    r << 7
}

fn bit_6_5(ck: CalibrationKind, range_id: u16) -> u16 {
    let r = match ck {
        CalibrationKind::CurrentAdc | CalibrationKind::ShuntResistance => range_id,
        CalibrationKind::VoltageDac => 0,
        _ => 0,
    };
    (r & 0b11) << 5
}

fn bit_4(ck: CalibrationKind, co: CalibrationObject) -> u16 {
    let r = match ck {
        CalibrationKind::CurrentAdc | CalibrationKind::VoltageDac => match co {
            CalibrationObject::Gain => 0,
            CalibrationObject::Offset => 1,
        },
        CalibrationKind::ShuntResistance => 0,
        _ => 0,
    };
    r << 4
}

fn bit_3_2_1(ch_idx: u16) -> u16 {
    (ch_idx & 7) << 1
}

impl AddressResolver for E192<El08b> {
    fn resolve(
        ck: CalibrationKind,
        range_id: u32,
        _: u32,
        co: CalibrationObject,
        ch_idx: u16,
        clk_div: Option<u16>,
    ) -> (Lsb<u16>, Msb<u16>) {
        let range_id = u16::try_from(range_id).expect("range_id does not fit in a u16");
        let b9 = bit_9(ck);
        let clk_div =
            clk_div.expect("Missing clock div, impossible to determine addresses without it");
        let b8_7 = bit_8_7(clk_div, ck);
        let b6_5 = bit_6_5(ck, range_id);
        let b4 = bit_4(ck, co);
        let b3_2_1 = bit_3_2_1(ch_idx);
        let address = b9 | b8_7 | b6_5 | b4 | b3_2_1;
        (Lsb(address | 1), Msb(address))
    }
}

#[cfg(test)]
mod e192_el08b_address_resolver_test {
    use super::*;

    const OTHER_KINDS: [CalibrationKind; 4] = [
        CalibrationKind::VoltageAdc,
        CalibrationKind::RsCorrection,
        CalibrationKind::CurrentDac,
        CalibrationKind::VoltageDac,
    ];

    #[test]
    fn test_bit_10_9() {
        assert_eq!(bit_9(CalibrationKind::CurrentAdc), 0);
        OTHER_KINDS
            .into_iter()
            .chain([CalibrationKind::ShuntResistance])
            .for_each(|ck| assert_eq!(bit_9(ck), 0x200));
    }

    #[test]
    fn test_bit_8_7_current_adc() {
        // clk_div is a 2 bit field
        [(0, 0), (1, 0x80), (2, 0x100), (3, 0x180)]
            .into_iter()
            .for_each(|(clk_div, res)| {
                assert_eq!(bit_8_7(clk_div, CalibrationKind::CurrentAdc), res)
            });
    }

    #[test]
    fn test_bit_8_7_shunt_resistance() {
        (0..4).for_each(|d| assert_eq!(bit_8_7(d, CalibrationKind::ShuntResistance), 0x80));
    }

    #[test]
    fn test_bit_8_7_all_others() {
        OTHER_KINDS
            .into_iter()
            .for_each(|ck| (0..4).for_each(|d| assert_eq!(bit_8_7(d, ck), 0)));
    }

    #[test]
    fn test_bit_6_5_current_adc_and_shunt() {
        // range_id is a 2 bit field
        [
            CalibrationKind::CurrentAdc,
            CalibrationKind::ShuntResistance,
        ]
        .into_iter()
        .for_each(|ck| {
            [(0, 0), (1, 0x20), (2, 0x40), (3, 0x60)]
                .into_iter()
                .for_each(|(range_id, res)| assert_eq!(bit_6_5(ck, range_id), res));
        });
    }

    #[test]
    fn test_bit_6_5_all_others() {
        OTHER_KINDS
            .into_iter()
            .for_each(|ck| (0..4).for_each(|r| assert_eq!(bit_6_5(ck, r), 0)));
    }

    #[test]
    fn test_bit_4() {
        [CalibrationKind::CurrentAdc, CalibrationKind::VoltageDac]
            .into_iter()
            .for_each(|ck| {
                assert_eq!(bit_4(ck, CalibrationObject::Gain), 0);
                assert_eq!(bit_4(ck, CalibrationObject::Offset), 0x10);
            });
        assert_eq!(
            bit_4(CalibrationKind::ShuntResistance, CalibrationObject::Gain),
            0
        );
        assert_eq!(
            bit_4(CalibrationKind::ShuntResistance, CalibrationObject::Offset),
            0
        );
    }

    #[test]
    fn test_get_bit_3_2_1() {
        (0..8).for_each(|ch_idx| assert_eq!(bit_3_2_1(ch_idx), ch_idx << 1));
    }
}
