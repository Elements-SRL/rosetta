use crate::{
    calibration_kind::{CalibrationKind, CalibrationObject},
};

pub trait AddressResolver {
    type Address;

    fn resolve(
        ck: CalibrationKind,
        range_id: u32,
        sr_id: u32,
        co: CalibrationObject,
        ch_idx: u16,
    ) -> Self::Address;
}
