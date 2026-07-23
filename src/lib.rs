use crate::{
    address_resolver::AddressResolver,
    devices::{SupportedDevices, e192::E192, syncro::SyncroV1},
    models::Calibration,
    resolutions::ResolutionSearch,
    stone::Stone,
};
use e384_rust::device::Device;
use std::{fmt::Debug, path::Path};
pub mod address_resolver;
pub mod calibration_kind;
pub mod constants;
pub mod devices;
pub mod models;
pub mod resolutions;
pub mod stone;
pub mod util;
pub mod workspace;

/// Calibrates the connected device, then (when `unpack` is set) writes per-board files.
pub fn calibrate(
    device_id: &str,
    calib: Calibration,
    workspace: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dev = Device::connect(device_id)
        .map_err(|e| format!("failed to connect to device (error code {e:?})"))?;

    let di = dev
        .device_info()
        .map_err(|e| format!("failed to read device info ({e:?})"))?;

    let device = SupportedDevices::from_device_version_info(&di)
        .ok_or_else(|| format!("device version {di:?} is incompatible with Rosetta"))?;

    match device {
        SupportedDevices::SyncroV1 => {
            run_calib_ops(Stone::<SyncroV1>::new(calib, dev), workspace, device_id)
        }
        SupportedDevices::E192 => {
            run_calib_ops(Stone::<E192>::new(calib, dev), workspace, device_id)
        }
    }
}

pub fn run_calib_ops<D: AddressResolver + ResolutionSearch + Debug>(
    stone: Stone<D>,
    workspace: &Path,
    device_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    stone.apply_complete_calibration();
    workspace::unpack_boards(
        stone.calibration(),
        workspace,
        device_id,
        &workspace::read_mapper(workspace),
    )?;
    Ok(())
}
