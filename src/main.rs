use clap::Parser;
use e384_rust::device::Device;
use rosetta::{calibrate, models, workspace};
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

const DEFAULT_CALIB_FILE: &str = "calibration_file.toml";

#[derive(Parser)]
struct Cli {
    /// Path to the workspace folder. Must already exist. Rosetta looks here for the
    /// calibration file and the optional `mapper.csv`, and writes per-board output here.
    workspace: PathBuf,

    /// Name of the device to connect to (also used as the output folder name). If omitted
    /// while calibrating, you'll be prompted to pick from the detected devices. Required
    /// with --only-files.
    #[arg(short = 'd', long)]
    device: Option<String>,

    /// Name of the calibration file inside the workspace (not a path).
    #[arg(short = 'c', long, default_value = DEFAULT_CALIB_FILE)]
    calib: String,

    /// Only unpack the calibration into one TOML per board; do not calibrate. Runs fully
    /// offline (no device connection), so --device is required.
    #[arg(short = 'o', long)]
    only_files: bool,
}

fn prompt_choice(items: &[String], label: &str) -> usize {
    loop {
        println!("Select a {label} by number:");
        for (i, item) in items.iter().enumerate() {
            println!("  [{i}] {item}");
        }

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        match input.trim().parse::<usize>() {
            Ok(idx) if idx < items.len() => return idx,
            _ => println!("Invalid selection, try again."),
        }
    }
}

fn resolve_device(requested: Option<String>) -> String {
    let devices = Device::list_devices().unwrap_or_else(|e| {
        eprintln!("failed to list devices (error code {e:?})");
        std::process::exit(1);
    });

    if devices.is_empty() {
        eprintln!("no devices found");
        std::process::exit(1);
    }

    match requested {
        Some(name) => {
            if devices.contains(&name) {
                name
            } else {
                eprintln!("device '{name}' not found. Available devices:");
                for d in &devices {
                    eprintln!("  {d}");
                }
                std::process::exit(1);
            }
        }
        None => {
            let idx = prompt_choice(&devices, "device");
            devices[idx].clone()
        }
    }
}

/// Validates the workspace folder exists, exiting with a clear message otherwise.
fn resolve_workspace(workspace: PathBuf) -> PathBuf {
    if !workspace.is_dir() {
        eprintln!("workspace folder '{}' does not exist", workspace.display());
        std::process::exit(1);
    }
    workspace
}

/// Resolves the calibration file inside the workspace, exiting if it is missing.
fn resolve_calib_file(workspace: &Path, name: &str) -> PathBuf {
    let path = workspace.join(name);
    if !path.is_file() {
        eprintln!("calibration file '{name}' not found in workspace");
        std::process::exit(1);
    }
    path
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = resolve_workspace(cli.workspace);
    let calib_file = resolve_calib_file(&workspace, &cli.calib);
    let calib = models::read_calibrations(&calib_file)
        .map_err(|e| format!("failed to read calibration file '{}': {e}", cli.calib))?;

    if cli.only_files {
        let sn = cli.device.ok_or("--device is required with --only-files")?;
        workspace::unpack_boards(&calib, &workspace, &sn, &workspace::read_mapper(&workspace))?;
        return Ok(());
    }

    let device_id = resolve_device(cli.device);
    calibrate(&device_id, calib, &workspace)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    tracing::info!("app started");

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
