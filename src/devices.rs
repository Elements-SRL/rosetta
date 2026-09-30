use crate::constants::{E192_VERSION, PATCH_384_VERSION};
use e384_rust::device_info::DeviceVersionInfo;

pub mod e192;
pub mod syncro;

pub enum SupportedDevices {
    SyncroV1,
    E192El03c,
    E192El08b,
}

impl SupportedDevices {
    pub fn from_device_version_info(di: &DeviceVersionInfo) -> Option<Self> {
        match di.device_version {
            PATCH_384_VERSION => match di.device_sub_version {
                3 | 4 | 7 => match di.fw_major {
                    7 => Some(SupportedDevices::SyncroV1),
                    _ => None,
                },
                _ => None,
            },
            E192_VERSION => match di.device_sub_version {
                7 => Some(SupportedDevices::E192El03c),
                8 => Some(SupportedDevices::E192El08b),
                _ => None,
            },
            _ => None,
        }
    }
}
