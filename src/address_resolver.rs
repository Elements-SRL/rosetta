use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject},
    util::{Lsb, Msb},
};

pub trait AddressResolver {
    fn resolve(
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
        ch_idx: u16,
    ) -> (Lsb<u16>, Msb<u16>);
}
