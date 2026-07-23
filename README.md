# Rosetta

Calibrate e384-family devices (Syncro, E192) and export their calibration as one TOML file per board.

Rosetta reads a single calibration file describing every board's gains and offsets, writes those
values into a connected device's calibration memory, and splits the calibration into per-board
files named from an optional mapping. It can also run **fully offline** to only produce the
per-board files.

---

## Contents

- [How it works](#how-it-works)
- [Installation](#installation)
- [Workspace layout](#workspace-layout)
- [Usage](#usage)
  - [Modes](#modes)
  - [Command-line options](#command-line-options)
  - [Examples](#examples)
- [Input files](#input-files)
  - [Calibration file](#calibration-file)
  - [`mapper.csv`](#mappercsv)
- [Output](#output)
- [Logging](#logging)
- [Supported devices](#supported-devices)
- [Exit codes & errors](#exit-codes--errors)
- [Development](#development)
- [Architecture](#architecture)

---

## How it works

A **calibration file** (`calibration_file.toml` by default) contains, for every board, a set of
gains and offsets grouped by *calibration kind* (current/voltage ADC & DAC, shunt resistance, Rs
correction), *range*, and *sampling rate*. Rosetta:

1. Connects to the device (unless running offline) and detects its model from the firmware version.
2. Applies the calibration: for each board it selects the calibration RAM, scales every gain/offset
   to the device's fixed-point resolution, resolves the target RAM address, and writes it — then
   commits RAM to EEPROM.
3. **Exports** the calibration as one TOML file per board into a folder named after the device,
   using names from an optional `mapper.csv`.

Steps 1–2 need a device; step 3 is pure file I/O and can be run on its own with `--only-files`.

> **Note:** a normal calibration run *always* exports the per-board files as well. Exporting is not
> an opt-in flag — it is part of every calibration.

## Installation

Rosetta is a Rust (edition 2024) binary. It depends on the local `e384_rust` crate (expected at
`../rust/e384_rust` relative to this repo).

```sh
# Build
cargo build --release

# The binary lands at target/release/rosetta
```

Terminal help is always available:

```sh
rosetta --help
```

## Workspace layout

Rosetta operates on a **workspace folder** that you pass as the only positional argument. The folder
**must already exist** (Rosetta never creates it). A typical workspace:

```
my_workspace/
├── calibration_file.toml   # required — the calibration to apply/export (name configurable)
├── mapper.csv              # optional — one board name per row
└── <device_sn>/            # created by Rosetta — per-board output files
    ├── 1_sn8.toml
    ├── 2_sn1.toml
    └── ...
```

## Usage

```
rosetta [OPTIONS] <WORKSPACE>
```

### Modes

| Mode | Command | Device needed? | What it does |
|------|---------|----------------|--------------|
| **Calibrate + export** (default) | `rosetta <ws> -d <sn>` | Yes | Connects, writes calibration to the device, then exports per-board files. |
| **Export only** (offline) | `rosetta <ws> -d <sn> --only-files` | No | Skips the device; only splits the calibration file into per-board files. `-d` is required (used as the output folder name). |

### Command-line options

| Option | Description |
|--------|-------------|
| `<WORKSPACE>` | Path to the workspace folder. Must already exist. |
| `-d`, `--device <DEVICE>` | Device serial / id. Used to connect **and** as the output folder name. If omitted while calibrating, Rosetta prompts you to pick from detected devices. **Required with `--only-files`.** |
| `-c`, `--calib <CALIB>` | Name (not a path) of the calibration file inside the workspace. Default: `calibration_file.toml`. |
| `-o`, `--only-files` | Only export the per-board files; do not calibrate. Runs fully offline. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

### Examples

Every command below is copy-pastable. They assume you are in the repo root and that your workspace
folder is `./my_workspace`. Adjust the path and `device_sn` to your setup.

#### Getting started

```sh
# Print help (full reference, examples, logging)
rosetta --help

# Short help summary
rosetta -h

# Print the version
rosetta --version
```

#### Running the built binary

```sh
# Build first, then run the release binary directly
cargo build --release

# Calibrate the connected device and export per-board files
./target/release/rosetta ./my_workspace -d device_sn

# Windows (PowerShell / cmd)
.\target\release\rosetta.exe .\my_workspace -d device_sn
```

#### Running via cargo (no separate build step)

```sh
# Everything after `--` is passed to Rosetta
cargo run --release -- ./my_workspace -d device_sn

# Debug build (faster to compile, slower to run)
cargo run -- ./my_workspace -d device_sn
```

#### Calibrate + export (default mode)

```sh
# Calibrate the connected device and export per-board files
rosetta ./my_workspace -d device_sn

# Pick the device interactively from the detected list (omit -d)
rosetta ./my_workspace

# Use a differently named calibration file inside the workspace
rosetta ./my_workspace -d device_sn -c my_calibration.toml

# Long-flag spelling
rosetta ./my_workspace --device device_sn --calib my_calibration.toml
```

#### Export only (offline, no device)

```sh
# Only split the calibration file into per-board files (no device connection)
rosetta ./my_workspace -d device_sn --only-files

# Short flag for --only-files
rosetta ./my_workspace -d device_sn -o

# Offline export with a custom calibration file name
rosetta ./my_workspace -d device_sn -c my_calibration.toml --only-files
```

#### Logging verbosity

```sh
# Default is info; no env var needed
rosetta ./my_workspace -d device_sn

# More detail
RUST_LOG=debug rosetta ./my_workspace -d device_sn

# Full per-call spans
RUST_LOG=trace rosetta ./my_workspace -d device_sn
```

On Windows PowerShell, set the env var separately:

```powershell
$env:RUST_LOG = "trace"; rosetta .\my_workspace -d device_sn
```

## Input files

### Calibration file

TOML describing shared sampling rates and one entry per board. Abbreviated shape:

```toml
[[sampling_rates]]
name = "slow"
id = 0
values = [5.0, 10.0, 20.0]
commlib_indexes = [0, 1, 2]

[[boards]]
board_number = 0

[[boards.current_adc]]          # calibration kind
range_name = "10nA"
range_id = 0

[[boards.current_adc.sampling_rates]]
sr_id = 0
# clk_div = 1                   # optional
[boards.current_adc.sampling_rates.calibrations]
gains   = [1.28, 1.27, ...]     # one value per channel
offsets = [-2.85e-9, ...]       # one value per channel
```

**Calibration kinds** (each is an optional array of range blocks on a board):
`current_adc`, `current_dac`, `voltage_adc`, `voltage_dac`, `shunt_resistance`, `rs_correction`.
A board may omit any kind it does not have — missing kinds default to empty and are skipped.

### `mapper.csv`

Optional file in the workspace root. **One board name per row**, positionally mapped to boards
(row 1 → first board, row 2 → second board, …). Only the first comma-separated field of each row is
used and it is trimmed, so trailing-comma rows are fine:

```csv
sn8,
sn1,
sn5,
```

If `mapper.csv` is absent (or unreadable), Rosetta does **not** fail — boards simply fall back to
numeric names. Extra rows beyond the board count are ignored; boards beyond the last row fall back
to numeric names.

## Output

Rosetta writes one TOML per board into `<workspace>/<device>/`, where `<device>` is the `-d` value.
Each file contains **only that board's block** (no shared `sampling_rates`), and re-parses cleanly
as a board. Filenames are:

- `<N>_<mapper-name>.toml` — 1-based index `N` prefixed to the board's `mapper.csv` name.
- `<N>.toml` — fallback when the board has no (non-empty) mapper entry.

Example, for a device `device_sn` with the `mapper.csv` above and 5 boards:

```
device_sn/
├── 1_sn8.toml
├── 2_sn1.toml
├── 3_sn5.toml
├── 4.toml
└── 5.toml
```

> Board files preserve every value to full `f64` precision; the textual float format may differ
> from the source file (e.g. scientific vs. decimal notation).

## Logging

Rosetta logs via [`tracing`]. Verbosity is set with the `RUST_LOG` environment variable and defaults
to `info`:

```sh
RUST_LOG=info   rosetta ./ws -d sn     # default
RUST_LOG=debug  rosetta ./ws -d sn
RUST_LOG=trace  rosetta ./ws -d sn     # includes per-call spans
```

During calibration Rosetta logs progress (board, calibration kind, range, sampling rate) and, at
`error` level, any individual RAM-write or device-command failure — these are logged but do **not**
abort the run.

[`tracing`]: https://docs.rs/tracing

## Supported devices

Detected from the device's version info:

| Model | `device_version` | `device_sub_version` |
|-------|------------------|----------------------|
| Syncro V1 | 15 | 7 |
| E192 | 13 | 7 |

Any other version combination is rejected with an "incompatible with Rosetta" error.

## Exit codes & errors

Rosetta exits non-zero (`1`) with a clear message on:

- **Workspace missing** — `workspace folder '<path>' does not exist`.
- **Calibration file missing** — `calibration file '<name>' not found in workspace`.
- **`--only-files` without `-d`** — `--device is required with --only-files`.
- **Calibration file unreadable / invalid TOML** — `failed to read calibration file '<name>': …`.
- **Device errors** — no devices found, requested device not found, connection/info failure, or an
  unsupported device version.

## Development

```sh
cargo build           # build
cargo test            # run the unit test suite
cargo clippy          # lint
cargo doc --open      # generate & open API docs (from the in-code doc comments)
```

The offline export path is exercised end-to-end without hardware — point Rosetta at a workspace
containing a calibration file and run with `--only-files`. The calibration path itself requires a
connected device.

## Architecture

| Module | Responsibility |
|--------|----------------|
| `main.rs` | CLI parsing, workspace/file validation, device selection prompt, mode dispatch. |
| `lib.rs` (`calibrate`, `run_calib_ops`) | Connects, detects the device, runs calibrate → export. |
| `stone` | `Stone<D>`, the calibration engine: scales values and writes calibration RAM/EEPROM. |
| `models` | Calibration file schema and TOML (de)serialization. |
| `devices` | Device detection; `syncro` and `e192` backends. |
| `resolutions` | Per-kind/range fixed-point resolutions used to scale gains/offsets. |
| `address_resolver` | Maps (kind, range, sampling rate, object, channel) to a RAM address. |
| `workspace` | `mapper.csv` parsing and per-board file export. |

The calibration engine is generic over a device backend `D` that implements both
`ResolutionSearch` (how to scale a value) and `AddressResolver` (where to write it), so adding a new
device is a matter of implementing those two traits.
