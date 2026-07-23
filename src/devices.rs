use e384_rust::device_info::DeviceVersionInfo;

use crate::{constants::PATCH_384, devices::SupportedDevices::SyncroV1};

pub mod e192;
pub mod syncro;

pub enum SupportedDevices {
    SyncroV1,
    E192,
}

impl SupportedDevices {
    pub fn from_device_version_info(di: &DeviceVersionInfo) -> Option<Self> {
        match di.device_version {
            PATCH_384 => Some(SyncroV1),
            _ => None,
        }
    }
}
