use crate::{
    address_resolver::AddressResolver,
    calibration_kind::{CalibrationKind, CalibrationObject},
    devices::e192::E192,
    util::{Lsb, Msb},
};

fn sr_id_to_clock_div(sr_id: u16) -> u16 {
    if sr_id < 4 { 0 } else { 1 }
}

fn get_bit_9(ck: CalibrationKind) -> u16 {
    if ck == CalibrationKind::CurrentAdc {
        0
    } else {
        0x200
    }
}

fn get_bit_8_7(ck: CalibrationKind, range_id: u16) -> u16 {
    let p = match ck {
        CalibrationKind::CurrentAdc => range_id << 7,
        _ => 0,
    };
    p & 0x180
}

fn get_bit_6_5(ck: CalibrationKind, sr_id: u16, clk_div: Option<u16>) -> u16 {
    let p = match ck {
        CalibrationKind::CurrentAdc => match clk_div {
        Some(ck) => ck << 5,
        _ => sr_id_to_clock_div(sr_id) << 5,
        },
        _ => 0,
    };
    p & 0x60
}

fn get_bit_4(co: CalibrationObject) -> u16 {
    let p = if co == CalibrationObject::Gain { 0 } else { 1 }; 
    p << 4 & 0x10
}

fn get_bit_3_2_1(ch_idx: u16) -> u16 {
    ch_idx << 1 & 0xE
}

impl AddressResolver for E192 {
    fn resolve(
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
        ch_idx: u16,
        clk_div: Option<u16>,
    ) -> (Lsb<u16>, Msb<u16>) {
        let range_id = range_id as u16;
        let sr_id = sr_id as u16;
        let b9 = get_bit_9(ck);
        let b8_7 = get_bit_8_7(ck, range_id);
        let b6_5 = get_bit_6_5(ck, sr_id, clk_div);
        let b4 = get_bit_4(co);
        let b3_2_1 = get_bit_3_2_1(ch_idx);
        let address = b9 | b8_7 | b6_5 | b4 | b3_2_1;
        (Lsb(address | 1), Msb(address))
    }
}

#[cfg(test)]
mod e192_address_resolver_test {
    use crate::{
        calibration_kind::{CalibrationKind, CalibrationObject}, devices::e192::address_resolver::{get_bit_3_2_1, get_bit_4, get_bit_6_5, get_bit_8_7, get_bit_9},
    };

    #[test]
    fn test_get_bit_9() {
        assert_eq!(get_bit_9(CalibrationKind::CurrentAdc), 0);
        vec![
            CalibrationKind::VoltageAdc,
            CalibrationKind::ShuntResistance,
            CalibrationKind::RsCorrection,
            CalibrationKind::CurrentDac,
            CalibrationKind::VoltageDac,
        ]
        .into_iter()
        .for_each(|ck| assert_eq!(get_bit_9(ck), 0x200));
    }

    #[test]
    fn test_get_bit_8_7() {
        vec![(0, 0), (1, 0x80), (2, 0x100), (3, 0x180)]
            .into_iter()
            .for_each(|(range_id, res)| {
                assert_eq!(get_bit_8_7(CalibrationKind::CurrentAdc, range_id), res)
            });
        vec![
            CalibrationKind::VoltageAdc,
            CalibrationKind::ShuntResistance,
            CalibrationKind::RsCorrection,
            CalibrationKind::CurrentDac,
            CalibrationKind::VoltageDac,
        ]
        .into_iter()
        .for_each(|ck| (0..16).into_iter().for_each(|r_id|assert_eq!(get_bit_8_7(ck, r_id), 0x00)));
    }

    #[test]
    fn test_get_bit_6_5_current_adc() {
        (0..7)
        .map(|i| (i, if i < 4 {0} else {0x20}))
        .for_each(|(i,r)| assert_eq!(get_bit_6_5(CalibrationKind::CurrentAdc, i, None), r));
    }

    #[test]
    fn test_get_bit_6_5_current_adc_ck_div() {
        vec![0, 0x20, 0x40, 0x60]
        .into_iter()
        .enumerate()
        .map(|(clk_div, res)| (clk_div as u16, res))
        .for_each(|(clk_div, res)| {
            (0..7)
                .for_each(|sr_id| assert_eq!(get_bit_6_5(CalibrationKind::CurrentAdc, sr_id, Some(clk_div)), res));
            })
    }

    #[test]
    fn test_get_bit_6_5_all_others() {
        vec![
            CalibrationKind::VoltageAdc,
            CalibrationKind::ShuntResistance,
            CalibrationKind::RsCorrection,
            CalibrationKind::CurrentDac,
            CalibrationKind::VoltageDac,
        ]
        .into_iter()
        .for_each(|ck| {
            (0..4).for_each(|clk_div|{
             (0..7)
                .for_each(|sr_id| {
                    assert_eq!(get_bit_6_5(ck, sr_id, Some(clk_div)), 0);
                    assert_eq!(get_bit_6_5(ck, sr_id, None), 0);
                });
            });
        });
        
    }
   
    #[test]
    fn test_get_bit_4() {
        assert_eq!(get_bit_4(CalibrationObject::Gain), 0);
        assert_eq!(get_bit_4(CalibrationObject::Offset), 0x10);
    }

    #[test]
    fn test_get_bit_3_2_1() {
        (0..8).for_each(|ch_idx| assert_eq!(get_bit_3_2_1(ch_idx), ch_idx << 1));
    }
}
