use clap::Parser;
use e384_rust::device::Device;
use rosetta::{
    devices::{SupportedDevices, syncro::SyncroV1},
    stele::Stele,
};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Cli {
    /// Name of the device to connect to. If omitted, you'll be prompted to pick from
    /// the list of detected devices.
    #[arg(short = 'd', long)]
    device: Option<String>,

    /// Path to the calibration TOML file. If omitted, the current directory is
    /// searched for a .toml file.
    #[arg(short = 'c', long)]
    calib_path: Option<PathBuf>,
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

fn resolve_calib_path(requested: Option<PathBuf>) -> PathBuf {
    if let Some(path) = requested {
        return path;
    }

    let cwd = std::env::current_dir().expect("failed to read current directory");
    let mut candidates: Vec<String> = std::fs::read_dir(&cwd)
        .expect("failed to read current directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    candidates.sort();

    match candidates.len() {
        0 => {
            eprintln!("no .toml file found in current directory, pass --calib-path");
            std::process::exit(1);
        }
        1 => cwd.join(&candidates[0]),
        _ => {
            let idx = prompt_choice(&candidates, "calibration file");
            cwd.join(&candidates[idx])
        }
    }
}

fn run(device_id: &str, calib_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let dev = Device::connect(device_id)
        .map_err(|e| format!("failed to connect to device (error code {e:?})"))?;

    let di = dev
        .device_info()
        .map_err(|e| format!("failed to read device info ({e:?})"))?;

    let device = SupportedDevices::from_device_version_info(&di)
        .ok_or_else(|| format!("device version {di:?} is incompatible with Rosetta"))?;

    match device {
        SupportedDevices::SyncroV1 | SupportedDevices::E192 => {
            let mut syncro = Stele::<SyncroV1>::new(calib_path, dev)?;
            syncro.apply_complete_calibration();
        }
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    tracing::info!("app started");

    let cli = Cli::parse();
    let device_id = resolve_device(cli.device);
    let calib_path = resolve_calib_path(cli.calib_path);

    if let Err(e) = run(&device_id, calib_path) {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
