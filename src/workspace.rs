use std::{
    io,
    path::{Path, PathBuf},
};

use crate::models::Calibration;

/// Name of the optional board-name mapping file in the workspace root.
pub const MAPPER_FILE: &str = "mapper.csv";
/// Folder inside the workspace that holds calibration snapshots and reset backups.
pub const ARCHIVE_DIR: &str = "archive";

/// Files written by [`init_workspace`]: `(file name, contents)`.
const SAMPLE_FILES: [(&str, &str); 3] = [
    ("README.md", include_str!("assets/workspace/README.md")),
    (MAPPER_FILE, include_str!("assets/workspace/mapper.csv")),
    (
        "calibration_file.example.toml",
        include_str!("assets/workspace/calibration_file.example.toml"),
    ),
];

/// Creates `workspace` (if needed) and writes every sample file that is missing.
///
/// Existing files are never overwritten. Returns the names of the files it created.
pub fn init_workspace(workspace: &Path) -> io::Result<Vec<&'static str>> {
    std::fs::create_dir_all(workspace)?;
    let mut created = Vec::new();
    for (name, contents) in SAMPLE_FILES {
        let path = workspace.join(name);
        if !path.exists() {
            std::fs::write(&path, contents)?;
            created.push(name);
        }
    }
    Ok(created)
}

/// Every top-level entry of the workspace except the archive folder, sorted.
pub fn workspace_entries(workspace: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(workspace)? {
        let path = entry?.path();
        if path.file_name() != Some(ARCHIVE_DIR.as_ref()) {
            entries.push(path);
        }
    }
    entries.sort();
    Ok(entries)
}

/// Moves every workspace entry (except the archive) into `archive/<stamp>_reset/`, then
/// recreates the sample files.
///
/// Returns the backup folder, or `None` if there was nothing to move.
pub fn reset_workspace(workspace: &Path, stamp: &str) -> io::Result<Option<PathBuf>> {
    let entries = workspace_entries(workspace)?;
    let mut backup = None;
    if !entries.is_empty() {
        let dest = workspace.join(ARCHIVE_DIR).join(format!("{stamp}_reset"));
        std::fs::create_dir_all(&dest)?;
        for entry in &entries {
            // Entries come from read_dir, so they always have a file name.
            std::fs::rename(entry, dest.join(entry.file_name().unwrap()))?;
        }
        backup = Some(dest);
    }
    init_workspace(workspace)?;
    Ok(backup)
}

/// Copies what a calibration run used and produced into `archive/<stamp>_<sn>/`: the
/// calibration file, `mapper.csv` (if present) and the board files, under `<sn>/` just like
/// in the workspace.
///
/// Returns the snapshot folder.
pub fn archive_calibration(
    workspace: &Path,
    sn: &str,
    stamp: &str,
    calib_file: &Path,
    board_files: &[PathBuf],
) -> io::Result<PathBuf> {
    let dest = workspace.join(ARCHIVE_DIR).join(format!("{stamp}_{sn}"));
    let boards_dest = dest.join(sn);
    std::fs::create_dir_all(&boards_dest)?;

    let mut copies = vec![(calib_file.to_path_buf(), dest.clone())];
    let mapper = workspace.join(MAPPER_FILE);
    if mapper.is_file() {
        copies.push((mapper, dest.clone()));
    }
    copies.extend(board_files.iter().map(|f| (f.clone(), boards_dest.clone())));

    for (src, dir) in copies {
        let name = src.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("not a file: {}", src.display()),
            )
        })?;
        std::fs::copy(&src, dir.join(name))?;
    }
    Ok(dest)
}

/// Reads the optional `mapper.csv` from the workspace root.
///
/// Each non-comment line maps positionally to a board (line 1 -> board 1, ...). Lines
/// starting with `#` are skipped and do not count. Only the first comma-separated field of
/// each line is used and it is trimmed, so trailing-comma rows like `sn8,` yield `sn8`. A
/// missing (or unreadable) file simply yields an empty vec: mapping is optional and must
/// never fail the run.
pub fn read_mapper(workspace: &Path) -> Vec<String> {
    let path = workspace.join(MAPPER_FILE);
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .map(|line| line.split(',').next().unwrap_or("").trim().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Unpacks the calibration into one TOML file per board under `workspace/<sn>/`.
///
/// Files are named `<mapper_row>.toml`, falling back to `<N>.toml` (1-based `N`) when there
/// is no mapper row for that board (absent mapper, fewer rows than boards, or an empty row).
/// Each file contains only the [`Board`](crate::models::Board) block. Returns the written
/// paths in board order.
pub fn unpack_boards(
    calib: &Calibration,
    workspace: &Path,
    sn: &str,
    mapper: &[String],
) -> io::Result<Vec<PathBuf>> {
    let out_dir = workspace.join(sn);
    std::fs::create_dir_all(&out_dir)?;

    let mut written = Vec::with_capacity(calib.boards.len());
    for (i, board) in calib.boards.iter().enumerate() {
        let n = i + 1;
        let file_name = match mapper.get(i).filter(|name| !name.is_empty()) {
            Some(name) => format!("{name}.toml"),
            None => format!("{n}.toml"),
        };
        let contents = toml::to_string_pretty(board)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let dest = out_dir.join(&file_name);
        std::fs::write(&dest, contents)?;
        tracing::info!("wrote {}", dest.display());
        written.push(dest);
    }

    Ok(written)
}

#[cfg(test)]
mod workspace_tests {
    use super::{
        ARCHIVE_DIR, archive_calibration, init_workspace, read_mapper, reset_workspace,
        unpack_boards,
    };
    use crate::models::{Board, Calibration, read_calibrations};
    use std::{fs, path::PathBuf};

    fn fresh_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn read_mapper_absent_is_empty() {
        let dir = fresh_dir("rosetta_mapper_absent_test");
        assert!(read_mapper(&dir).is_empty());
    }

    #[test]
    fn read_mapper_trailing_comma_and_trim() {
        let dir = fresh_dir("rosetta_mapper_trailing_test");
        fs::write(dir.join("mapper.csv"), "sn8,\n sn1 ,extra\nsn5\n").unwrap();
        assert_eq!(read_mapper(&dir), vec!["sn8", "sn1", "sn5"]);
    }

    #[test]
    fn read_mapper_skips_comments_but_keeps_empty_rows() {
        let dir = fresh_dir("rosetta_mapper_comments_test");
        fs::write(
            dir.join("mapper.csv"),
            "# header\nsn8\n  # indented\n\nsn5\n",
        )
        .unwrap();
        assert_eq!(read_mapper(&dir), vec!["sn8", "", "sn5"]);
    }

    #[test]
    fn init_writes_samples_and_keeps_existing_files() {
        let dir = fresh_dir("rosetta_init_test").join("nested");
        let created = init_workspace(&dir).unwrap();
        assert_eq!(created.len(), 3);

        // The sample mapper is all comments, so it maps nothing.
        assert!(read_mapper(&dir).is_empty());
        // The example calibration must stay a valid calibration file.
        let calib = read_calibrations(dir.join("calibration_file.example.toml")).unwrap();
        assert_eq!(calib.boards.len(), 2);

        fs::write(dir.join("mapper.csv"), "mine\n").unwrap();
        assert!(init_workspace(&dir).unwrap().is_empty());
        assert_eq!(
            fs::read_to_string(dir.join("mapper.csv")).unwrap(),
            "mine\n"
        );
    }

    #[test]
    fn reset_moves_everything_but_the_archive() {
        let dir = fresh_dir("rosetta_reset_test");
        init_workspace(&dir).unwrap();
        fs::write(dir.join("calibration_file.toml"), "x").unwrap();
        fs::create_dir_all(dir.join("SN1")).unwrap();
        fs::write(dir.join("SN1").join("1.toml"), "y").unwrap();
        fs::create_dir_all(dir.join(ARCHIVE_DIR).join("old")).unwrap();

        let backup = reset_workspace(&dir, "STAMP").unwrap().unwrap();
        assert_eq!(backup, dir.join(ARCHIVE_DIR).join("STAMP_reset"));
        assert!(backup.join("calibration_file.toml").is_file());
        assert!(backup.join("SN1").join("1.toml").is_file());
        assert!(backup.join("mapper.csv").is_file());
        // Previous archive content is untouched and not nested into the backup.
        assert!(dir.join(ARCHIVE_DIR).join("old").is_dir());
        assert!(!backup.join(ARCHIVE_DIR).exists());

        // Workspace holds only fresh samples (plus the archive).
        assert!(!dir.join("calibration_file.toml").exists());
        assert!(!dir.join("SN1").exists());
        assert!(dir.join("mapper.csv").is_file());
        assert!(dir.join("README.md").is_file());
    }

    #[test]
    fn reset_on_empty_workspace_creates_no_backup() {
        let dir = fresh_dir("rosetta_reset_empty_test");
        assert!(reset_workspace(&dir, "STAMP").unwrap().is_none());
        assert!(dir.join("README.md").is_file());
    }

    #[test]
    fn archive_calibration_snapshots_inputs_and_outputs() {
        let dir = fresh_dir("rosetta_archive_test");
        let calib_file = dir.join("calibration_file.toml");
        fs::write(&calib_file, "c").unwrap();
        fs::write(dir.join("mapper.csv"), "alpha\n").unwrap();
        fs::create_dir_all(dir.join("SN9")).unwrap();
        let board = dir.join("SN9").join("alpha.toml");
        fs::write(&board, "b").unwrap();

        let dest = archive_calibration(&dir, "SN9", "STAMP", &calib_file, &[board]).unwrap();
        assert_eq!(dest, dir.join(ARCHIVE_DIR).join("STAMP_SN9"));
        assert_eq!(
            fs::read_to_string(dest.join("calibration_file.toml")).unwrap(),
            "c"
        );
        assert_eq!(
            fs::read_to_string(dest.join("mapper.csv")).unwrap(),
            "alpha\n"
        );
        assert_eq!(
            fs::read_to_string(dest.join("SN9").join("alpha.toml")).unwrap(),
            "b"
        );
    }

    #[test]
    fn unpack_names_files_and_round_trips_as_board() {
        let dir = fresh_dir("rosetta_unpack_test");

        // Two boards; mapper only names the first, so the second falls back to "2.toml".
        let calib: Calibration = toml::from_str(
            r#"
            [[sampling_rates]]
            name = "slow"
            id = 0
            values = [1.0]
            commlib_indexes = [0]

            [[boards]]
            board_number = 7
            [[boards.current_adc]]
            range_name = "10nA"
            range_id = 0
            [[boards.current_adc.sampling_rates]]
            sr_id = 0
            [boards.current_adc.sampling_rates.calibrations]
            gains = [1.5]
            offsets = [-2.0e-9]

            [[boards]]
            board_number = 9
            "#,
        )
        .unwrap();

        let written = unpack_boards(&calib, &dir, "SN123", &["alpha".to_string()]).unwrap();

        let out = dir.join("SN123");
        let mapped = out.join("alpha.toml");
        let fallback = out.join("2.toml");
        assert_eq!(written, vec![mapped.clone(), fallback.clone()]);
        // No leading index prefix on mapped files (e.g. old "1_alpha.toml" form).
        assert!(!out.join("1_alpha.toml").exists());

        // Each produced file must parse back as a Board (Serialize/Deserialize round-trip).
        let b1: Board = toml::from_str(&fs::read_to_string(&mapped).unwrap()).unwrap();
        assert_eq!(b1.board_number, 7);
        assert_eq!(b1.current_adc.len(), 1);
        // Empty kinds are skipped on serialize and default back to empty on read.
        assert!(b1.voltage_dac.is_empty());

        let b2: Board = toml::from_str(&fs::read_to_string(&fallback).unwrap()).unwrap();
        assert_eq!(b2.board_number, 9);
        assert!(b2.current_adc.is_empty());
    }
}
