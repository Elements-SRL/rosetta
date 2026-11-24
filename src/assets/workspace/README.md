# Rosetta workspace

This folder was created by `rosetta init`. Rosetta reads its inputs from here and writes its
outputs here.

## What to change before calibrating

1. **Add the real calibration** as `calibration_file.toml` in this folder.
   `calibration_file.example.toml` shows the format but has **fake values**. Rosetta never
   loads it by default. Use a different file name with `-c <name>`.
2. **Edit `mapper.csv`** to name the output files (optional). One name per line, matched to
   boards **by position**: the 1st name goes to the 1st `[[boards]]` block in the calibration
   file, the 2nd to the 2nd, and so on. Lines starting with `#` are ignored. Boards without a
   name get `1.toml`, `2.toml`, ...
3. **Leave `archive/` alone.** Rosetta manages it (see below).

## Commands

| Command | What it does |
|---------|--------------|
| `rosetta calibrate` | Pick a connected device, write the calibration, export per-board files. |
| `rosetta calibrate -d <sn>` | Same, for the device with serial `<sn>`. |
| `rosetta calibrate -d <sn> -c other.toml` | Use another calibration file from this folder. |
| `rosetta export -d <name>` | Offline: only export per-board files into `<name>/`. No device needed. |
| `rosetta export -d demo -c calibration_file.example.toml` | Try the export with the example file. |
| `rosetta reset` | Move **everything** in this folder into `archive/` and recreate these sample files. Asks first. |
| `rosetta init` | Choose another workspace folder, or recreate missing sample files. |

Set `RUST_LOG=debug` (PowerShell: `$env:RUST_LOG = "debug"`) for more detailed logs.

## Layout

```
workspace/
├── README.md                       # this file
├── calibration_file.toml           # you add this: the real calibration
├── calibration_file.example.toml   # format example (fake values)
├── mapper.csv                      # board names, by position
├── <sn>/                           # per-board output, one TOML per board
└── archive/
    ├── <date>_<time>UTC_<sn>/      # snapshot of every successful calibration
    └── <date>_<time>UTC_reset/     # what was in the folder before a `rosetta reset`
```

Every successful `rosetta calibrate` copies the calibration file, `mapper.csv` and the board
files it just wrote into `archive/<timestamp>_<sn>/`. This keeps a record of what was written to
each device and when. Timestamps are in UTC.
