//! Rosetta calibrates e384-family devices (Syncro, E192) and exports their calibration
//! as one TOML file per board.
//!
//! # Workflow
//!
//! A calibration file ([`models::Calibration`]) describes every board's gains and offsets
//! per calibration kind, range, and sampling rate. [`calibrate`] connects to a device,
//! writes those values into the device's calibration RAM/EEPROM (see
//! [`stone::Stone::apply_complete_calibration`]), then unpacks the same data into per-board
//! files ([`workspace::unpack_boards`]). The offline path skips the device and only unpacks.
//!
//! # Module map
//!
//! * [`models`] — calibration file schema and TOML (de)serialization.
//! * [`stone`] — [`stone::Stone`], the calibration engine, generic over a device backend.
//! * [`devices`] — device detection and the per-device backends ([`devices::syncro`],
//!   [`devices::e192`]) implementing [`resolutions::ResolutionSearch`] and
//!   [`address_resolver::AddressResolver`].
//! * [`resolutions`] / [`address_resolver`] — value scaling and RAM address mapping.
//! * [`workspace`] — `mapper.csv` parsing and per-board file export.

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

/// Connects to `device_id`, applies the full calibration, then exports one TOML per board
/// into `workspace/<device_id>/`.
///
/// The device model is detected from its version info; unsupported devices return an error.
/// Returns an error if the device cannot be connected to or its info cannot be read; per-RAM
/// write failures are logged (via `tracing`) rather than aborting the run.
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

/// Runs the calibration then exports per-board files, for an already-built [`Stone`].
///
/// Shared by every supported-device branch of [`calibrate`] so the ordering
/// (calibrate → unpack) lives in exactly one place.
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
