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
- [Getting started](#getting-started)
- [Workspace](#workspace)
- [Usage](#usage)
  - [Commands](#commands)
  - [Examples](#examples)
- [Input files](#input-files)
  - [Calibration file](#calibration-file)
  - [`mapper.csv`](#mappercsv)
- [Output](#output)
- [Archive](#archive)
- [Logging](#logging)
- [Supported devices](#supported-devices)
- [Exit codes & errors](#exit-codes--errors)
- [Development](#development)
- [Architecture](#architecture)

---

## How it works

A **calibration file** (`calibration_file.toml` by default) contains, for every board, a set of
gains and offsets grouped by *calibration kind* (current/voltage ADC & DAC, shunt resistance, Rs
correction), *range*, and *sampling rate*. `rosetta calibrate`:

1. Connects to the device and detects its model from the firmware version.
2. Applies the calibration: for each board it selects the calibration RAM, scales every gain/offset
   to the device's fixed-point resolution, resolves the target RAM address, and writes it, then
   commits RAM to EEPROM.
3. **Exports** the calibration as one TOML file per board into a folder named after the device,
   using names from an optional `mapper.csv`.
4. **Archives** a snapshot of the calibration file, `mapper.csv` and the board files into
   `archive/<timestamp>_<device>/`.

Step 3 is pure file I/O and can be run on its own, offline, with `rosetta export`.

## Installation

Rosetta is a Rust (edition 2024) binary. It depends on the local `e384_rust` crate (expected at
`../../wrappers/e384_rust` relative to this repo).

```sh
# Build
cargo build --release

# The binary lands at target/release/rosetta
```

On Windows, `build-installer.ps1` builds an Inno Setup installer (`setup.iss`) that can add
Rosetta to `PATH`.

Terminal help is always available:

```sh
rosetta --help
rosetta calibrate --help
```

## Getting started

```sh
rosetta init
```

`init` asks for the workspace folder, proposing `%USERPROFILE%\Documents\rosetta` (press Enter to
accept), creates it, and writes:

- `README.md`: what to change and which commands to run.
- `mapper.csv`: a fully commented example of the board-name mapping. It maps nothing until you edit it.
- `calibration_file.example.toml`: a small calibration with **fake values** that shows the format.

The chosen folder is saved in `%APPDATA%\rosetta\config.toml` and used by every other command.
Then put the real calibration in the workspace as `calibration_file.toml`, edit `mapper.csv`, and
run `rosetta calibrate`.

Run `init` again at any time to switch to another folder or to recreate missing sample files.
Existing files are never overwritten.

> The example calibration is deliberately **not** named `calibration_file.toml`, so its fake values
> cannot be written to a device by accident. Try it offline with
> `rosetta export -d demo -c calibration_file.example.toml`.

## Workspace

```
<workspace>/
├── README.md                       # written by `rosetta init`
├── calibration_file.toml           # you add this: the calibration to apply/export (name configurable)
├── calibration_file.example.toml   # format example, fake values
├── mapper.csv                      # optional: one board name per row
├── <device_sn>/                    # created by Rosetta: per-board output files
│   ├── sn8.toml
│   └── ...
└── archive/                        # created by Rosetta, see "Archive"
    ├── 2026-11-24_153012UTC_<device_sn>/
    └── 2026-11-24_160000UTC_reset/
```

## Usage

```
rosetta <COMMAND> [OPTIONS]
```

### Commands

| Command | Device needed? | What it does |
|---------|----------------|--------------|
| `init` | No | Choose the workspace folder and write the README and sample files. |
| `calibrate [-d <sn>] [-c <file>]` | Yes | Write the calibration to the device, export per-board files, archive a snapshot. Without `-d` you pick from the detected devices. |
| `export -d <name> [-c <file>]` | No | Offline: only split the calibration into per-board files under `<name>/`. |
| `reset [-y]` | No | Move **everything** in the workspace except `archive/` into `archive/<timestamp>_reset/`, then recreate the sample files. Lists what will be moved and asks for confirmation (`-y` skips it). |

Options:

| Option | Description |
|--------|-------------|
| `-d`, `--device` | Device serial / id. Used to connect **and** as the output folder name. Required for `export`. |
| `-c`, `--calib` | Name (not a path) of the calibration file inside the workspace. Default: `calibration_file.toml`. |
| `-y`, `--yes` | `reset` only: don't ask for confirmation. |

### Examples

```sh
# First time
rosetta init

# Calibrate the connected device and export per-board files
rosetta calibrate -d device_sn

# Pick the device interactively (omit -d)
rosetta calibrate

# Use a differently named calibration file inside the workspace
rosetta calibrate -d device_sn -c my_calibration.toml

# Offline: only split the calibration file into per-board files
rosetta export -d device_sn

# Try the export with the example file
rosetta export -d demo -c calibration_file.example.toml

# Start over: archive everything and recreate the sample files
rosetta reset

# Via cargo: everything after `--` is passed to Rosetta
cargo run --release -- calibrate -d device_sn
```

## Input files

### Calibration file

TOML describing shared sampling rates and one entry per board. `calibration_file.example.toml`
(written by `rosetta init`) is a complete, commented example. Abbreviated shape:

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
A board may omit any kind it does not have. Missing kinds default to empty and are skipped.

### `mapper.csv`

Optional file in the workspace root. **One board name per row**, mapped to boards by position
(row 1 → first `[[boards]]` block, row 2 → second, …). `board_number` is not used for the mapping.

- Lines starting with `#` are comments: skipped, and they don't count as a row.
- Only the first comma-separated field of each row is used, trimmed, so trailing-comma rows are fine.
- An empty row keeps that board's numeric name.

```csv
# names for SN123
sn8,
sn1,
sn5,
```

If `mapper.csv` is absent (or unreadable), Rosetta does **not** fail. Boards simply fall back to
numeric names. Extra rows beyond the board count are ignored; boards beyond the last row fall back
to numeric names.

## Output

Rosetta writes one TOML per board into `<workspace>/<device>/`, where `<device>` is the `-d` value
(or the device picked interactively). Each file contains **only that board's block** (no shared
`sampling_rates`), and re-parses cleanly as a board. Filenames are:

- `<mapper-name>.toml`: the board's `mapper.csv` name.
- `<N>.toml`: fallback when the board has no (non-empty) mapper entry.

Example, for a device `device_sn` with the `mapper.csv` above and 5 boards:

```
device_sn/
├── sn8.toml
├── sn1.toml
├── sn5.toml
├── 4.toml
└── 5.toml
```

> Board files preserve every value to full `f64` precision. The textual float format may differ
> from the source file (e.g. scientific vs. decimal notation).

## Archive

`<workspace>/archive/` is managed by Rosetta and is never moved by `reset`.

- **After every successful `calibrate`**, Rosetta copies the calibration file, `mapper.csv` (if
  present) and the board files written in that run into `archive/<timestamp>_<device>/`, laid out
  like the workspace. This keeps a record of what was written to each device and when.
- **`reset`** moves everything else in the workspace into `archive/<timestamp>_reset/`.

Timestamps are UTC, formatted `YYYY-MM-DD_HHMMSSUTC`. Nothing is deleted automatically: remove old
archive folders by hand when you no longer need them.

## Logging

Rosetta logs via [`tracing`]. Verbosity is set with the `RUST_LOG` environment variable and defaults
to `info`:

```sh
RUST_LOG=info   rosetta calibrate -d sn     # default
RUST_LOG=debug  rosetta calibrate -d sn
RUST_LOG=trace  rosetta calibrate -d sn     # includes per-call spans
```

On Windows PowerShell, set the env var separately:

```powershell
$env:RUST_LOG = "trace"; rosetta calibrate -d sn
```

Every command that uses the workspace logs its path first. During calibration Rosetta logs progress
(board, calibration kind, range, sampling rate) and, at `error` level, any individual RAM-write or
device-command failure. These are logged but do **not** abort the run.

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

- **No workspace configured**: `no workspace configured, run `rosetta init` first`.
- **Workspace folder gone**: `workspace folder '<path>' does not exist, run `rosetta init` to recreate it`.
- **Calibration file missing**: `calibration file '<name>' not found in workspace '<path>'`.
- **Calibration file unreadable / invalid TOML**: `failed to read calibration file '<name>': …`.
- **Device errors**: no devices found, requested device not found, connection/info failure, or an
  unsupported device version.

## Development

```sh
cargo build           # build
cargo test            # run the unit test suite
cargo clippy          # lint
cargo doc --open      # generate & open API docs (from the in-code doc comments)
```

The offline path is exercised end-to-end without hardware: `rosetta init`, then
`rosetta export -d demo -c calibration_file.example.toml`. To keep your real config untouched while
testing, point `APPDATA` and `USERPROFILE` at a scratch folder first. The calibration path itself
requires a connected device.

The workspace sample files live in `src/assets/workspace/` and are embedded in the binary with
`include_str!`.

## Architecture

| Module | Responsibility |
|--------|----------------|
| `main.rs` | CLI subcommands, prompts, workspace/file validation, device selection, dispatch. |
| `lib.rs` (`calibrate`, `run_calib_ops`) | Connects, detects the device, runs calibrate → export. |
| `config` | Saved workspace location (`%APPDATA%\rosetta\config.toml`) and the default proposal. |
| `workspace` | Workspace init/reset, `mapper.csv` parsing, per-board export, calibration archive. |
| `stone` | `Stone<D>`, the calibration engine: scales values and writes calibration RAM/EEPROM. |
| `models` | Calibration file schema and TOML (de)serialization. |
| `devices` | Device detection; `syncro` and `e192` backends. |
| `resolutions` | Per-kind/range fixed-point resolutions used to scale gains/offsets. |
| `address_resolver` | Maps (kind, range, sampling rate, object, channel) to a RAM address. |
| `util` | Byte splitting, resolution math, UTC timestamp formatting. |

The calibration engine is generic over a device backend `D` that implements both
`ResolutionSearch` (how to scale a value) and `AddressResolver` (where to write it), so adding a new
device is a matter of implementing those two traits.
