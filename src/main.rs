use clap::{Parser, Subcommand};
use e384_rust::device::Device;
use rosetta::{calibrate, config, models, util::utc_stamp, workspace};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing_subscriber::EnvFilter;

const DEFAULT_CALIB_FILE: &str = "calibration_file.toml";

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Rosetta: calibrate e384 devices and export per-board calibration files.
///
/// Rosetta works in a *workspace* folder chosen once with `rosetta init`. The workspace holds
/// the calibration file (default `calibration_file.toml`), an optional `mapper.csv` naming
/// each board, the per-board output folders and an `archive/` with a snapshot of every
/// calibration. `rosetta init` also writes a README explaining what to change.
#[derive(Parser)]
#[command(
    name = "rosetta",
    version,
    about = "Write calibrations to eeprom on elements devices and export per-board calibration files",
    after_help = "EXAMPLES:\n  \
        # First time: choose the workspace folder and create the sample files:\n  \
        rosetta init\n\n  \
        # Same, without the prompt:\n  \
        rosetta init D:\\calibrations\\rosetta\n\n  \
        # Calibrate the connected device and export per-board files:\n  \
        rosetta calibrate -d device_sn\n\n  \
        # Same, but pick the device interactively (omit -d):\n  \
        rosetta calibrate\n\n  \
        # Offline: only split the calibration file into per-board files:\n  \
        rosetta export -d device_sn\n\n  \
        # Move everything in the workspace into archive/ and start fresh:\n  \
        rosetta reset\n\n\
        LOGGING:\n  \
        Verbosity is controlled by the RUST_LOG env var (default: info).\n  \
        e.g. RUST_LOG=trace rosetta calibrate -d device_sn"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Choose the workspace folder and fill it with a README and sample files. Run it again to
    /// change folder or to recreate missing sample files (existing files are kept).
    Init {
        /// Workspace folder. If omitted, you'll be asked (Enter accepts the proposed one).
        path: Option<PathBuf>,
    },

    /// Write the calibration to a device, export one TOML per board into `<device>/`, and
    /// snapshot the inputs and outputs into `archive/<timestamp>_<device>/`.
    Calibrate {
        /// Device serial to connect to (also the output folder name). If omitted, you'll be
        /// prompted to pick from the detected devices.
        #[arg(short = 'd', long)]
        device: Option<String>,

        /// Name of the calibration file inside the workspace (not a path).
        #[arg(short = 'c', long, default_value = DEFAULT_CALIB_FILE)]
        calib: String,
    },

    /// Offline: only export one TOML per board into `<device>/`. No device is contacted.
    Export {
        /// Output folder name, normally the device serial.
        #[arg(short = 'd', long)]
        device: String,

        /// Name of the calibration file inside the workspace (not a path).
        #[arg(short = 'c', long, default_value = DEFAULT_CALIB_FILE)]
        calib: String,
    },

    /// Move EVERYTHING in the workspace (except `archive/`) into `archive/<timestamp>_reset/`,
    /// then recreate the sample files. Asks for confirmation.
    Reset {
        /// Don't ask for confirmation.
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Prints `prompt` and returns the trimmed line the user typed ("" on EOF).
fn ask(prompt: &str) -> Res<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
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

fn resolve_device(requested: Option<String>) -> Res<String> {
    let devices =
        Device::list_devices().map_err(|e| format!("failed to list devices (error code {e:?})"))?;

    if devices.is_empty() {
        return Err("no devices found".into());
    }

    match requested {
        Some(name) => {
            if devices.contains(&name) {
                Ok(name)
            } else {
                let mut msg = format!("device '{name}' not found. Available devices:");
                for d in &devices {
                    msg.push_str(&format!("\n  {d}"));
                }
                Err(msg.into())
            }
        }
        None => {
            let idx = prompt_choice(&devices, "device");
            Ok(devices[idx].clone())
        }
    }
}

/// Returns the configured workspace, which must exist.
fn load_workspace() -> Res<PathBuf> {
    let cfg = config::load(&config::config_path()?)?
        .ok_or("no workspace configured, run `rosetta init` first")?;
    if !cfg.workspace.is_dir() {
        return Err(format!(
            "workspace folder '{}' does not exist, run `rosetta init` to recreate it",
            cfg.workspace.display()
        )
        .into());
    }
    tracing::info!("workspace: {}", cfg.workspace.display());
    Ok(cfg.workspace)
}

/// Resolves the calibration file inside the workspace.
fn resolve_calib_file(workspace: &Path, name: &str) -> Res<PathBuf> {
    let path = workspace.join(name);
    if !path.is_file() {
        return Err(format!(
            "calibration file '{name}' not found in workspace '{}'",
            workspace.display()
        )
        .into());
    }
    Ok(path)
}

fn read_calib(workspace: &Path, name: &str) -> Res<(PathBuf, models::Calibration)> {
    let path = resolve_calib_file(workspace, name)?;
    let calib = models::read_calibrations(&path)
        .map_err(|e| format!("failed to read calibration file '{name}': {e}"))?;
    Ok((path, calib))
}

fn init(path: Option<PathBuf>) -> Res<()> {
    let cfg_path = config::config_path()?;
    let chosen = match path {
        Some(p) => p,
        None => {
            let proposed = config::load(&cfg_path)?
                .map(|c| c.workspace)
                .unwrap_or_else(config::default_workspace);
            let answer = ask(&format!("Workspace folder [{}]: ", proposed.display()))?;
            if answer.is_empty() {
                proposed
            } else {
                PathBuf::from(answer)
            }
        }
    };
    let workspace = std::path::absolute(&chosen)?;

    let created = workspace::init_workspace(&workspace)?;
    config::save(
        &cfg_path,
        &config::Config {
            workspace: workspace.clone(),
        },
    )?;

    println!("Workspace: {}", workspace.display());
    for name in &created {
        println!("  created {name}");
    }
    if created.is_empty() {
        println!("  all sample files already present, nothing overwritten");
    }
    println!("Next: read README.md in the workspace, add calibration_file.toml, edit mapper.csv.");
    Ok(())
}

fn reset(yes: bool) -> Res<()> {
    let ws = load_workspace()?;
    let entries = workspace::workspace_entries(&ws)?;
    if entries.is_empty() {
        workspace::init_workspace(&ws)?;
        println!("Workspace was empty; sample files recreated.");
        return Ok(());
    }

    let stamp = utc_stamp(SystemTime::now());
    println!(
        "This empties the workspace {}.\nThese {} item(s) will be MOVED into {}:",
        ws.display(),
        entries.len(),
        Path::new(workspace::ARCHIVE_DIR)
            .join(format!("{stamp}_reset"))
            .display()
    );
    for e in &entries {
        println!("  {}", e.file_name().unwrap_or_default().to_string_lossy());
    }
    println!("The sample files will then be recreated.");

    if !yes && !ask("Continue? [y/N]: ")?.eq_ignore_ascii_case("y") {
        println!("Aborted, nothing changed.");
        return Ok(());
    }

    if let Some(backup) = workspace::reset_workspace(&ws, &stamp)? {
        println!("Done. Previous contents are in {}", backup.display());
    }
    Ok(())
}

fn run(cli: Cli) -> Res<()> {
    match cli.command {
        Command::Init { path } => init(path),
        Command::Reset { yes } => reset(yes),
        Command::Export { device, calib } => {
            let ws = load_workspace()?;
            let (_, calib) = read_calib(&ws, &calib)?;
            workspace::unpack_boards(&calib, &ws, &device, &workspace::read_mapper(&ws))?;
            Ok(())
        }
        Command::Calibrate { device, calib } => {
            let ws = load_workspace()?;
            let (calib_path, calib) = read_calib(&ws, &calib)?;
            let device_id = resolve_device(device)?;
            let board_files = calibrate(&device_id, calib, &ws)?;
            let snapshot = workspace::archive_calibration(
                &ws,
                &device_id,
                &utc_stamp(SystemTime::now()),
                &calib_path,
                &board_files,
            )?;
            tracing::info!("calibration archived in {}", snapshot.display());
            Ok(())
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}
