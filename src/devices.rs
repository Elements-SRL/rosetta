use crate::constants::{E192_VERSION, PATCH_384_VERSION};
use e384_rust::device_info::DeviceVersionInfo;

pub mod e192;
pub mod syncro;

pub enum SupportedDevices {
    SyncroV1,
    E192,
}

impl SupportedDevices {
    pub fn from_device_version_info(di: &DeviceVersionInfo) -> Option<Self> {
        match di.device_version {
            PATCH_384_VERSION => match di.device_sub_version {
                7 => Some(SupportedDevices::SyncroV1),
                _ => None,
            },
            E192_VERSION => match di.device_sub_version {
                7 => Some(SupportedDevices::E192),
                _ => None,
            },
            _ => None,
        }
    }
}
