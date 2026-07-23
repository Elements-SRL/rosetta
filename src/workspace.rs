use std::{io, path::Path};

use crate::models::Calibration;

/// Reads the optional `mapper.csv` from the workspace root.
///
/// Each line maps positionally to a board (line 1 -> board 1, ...). Only the first
/// comma-separated field of each line is used and it is trimmed, so trailing-comma rows
/// like `sn8,` yield `sn8`. A missing (or unreadable) file simply yields an empty vec —
/// mapping is optional and must never fail the run.
pub fn read_mapper(workspace: &Path) -> Vec<String> {
    let path = workspace.join("mapper.csv");
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(|line| line.split(',').next().unwrap_or("").trim().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Unpacks the calibration into one TOML file per board under `workspace/<sn>/`.
///
/// Files are named `<N>_<mapper_row>.toml` (1-based `N`), falling back to `<N>.toml` when
/// there is no mapper row for that board (absent mapper, fewer rows than boards, or an
/// empty row). Each file contains only the [`Board`](crate::models::Board) block.
pub fn unpack_boards(
    calib: &Calibration,
    workspace: &Path,
    sn: &str,
    mapper: &[String],
) -> io::Result<()> {
    let out_dir = workspace.join(sn);
    std::fs::create_dir_all(&out_dir)?;

    for (i, board) in calib.boards.iter().enumerate() {
        let n = i + 1;
        let file_name = match mapper.get(i).filter(|name| !name.is_empty()) {
            Some(name) => format!("{n}_{name}.toml"),
            None => format!("{n}.toml"),
        };
        let contents = toml::to_string_pretty(board)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let dest = out_dir.join(&file_name);
        std::fs::write(&dest, contents)?;
        tracing::info!("wrote {}", dest.display());
    }

    Ok(())
}

#[cfg(test)]
mod workspace_tests {
    use super::{read_mapper, unpack_boards};
    use crate::models::{Board, Calibration};
    use std::fs;

    #[test]
    fn read_mapper_absent_is_empty() {
        let dir = std::env::temp_dir().join("rosetta_mapper_absent_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(read_mapper(&dir).is_empty());
    }

    #[test]
    fn read_mapper_trailing_comma_and_trim() {
        let dir = std::env::temp_dir().join("rosetta_mapper_trailing_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("mapper.csv"), "sn8,\n sn1 ,extra\nsn5\n").unwrap();
        assert_eq!(read_mapper(&dir), vec!["sn8", "sn1", "sn5"]);
    }

    #[test]
    fn unpack_names_files_and_round_trips_as_board() {
        let dir = std::env::temp_dir().join("rosetta_unpack_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

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

        unpack_boards(&calib, &dir, "SN123", &["alpha".to_string()]).unwrap();

        let out = dir.join("SN123");
        let mapped = out.join("1_alpha.toml");
        let fallback = out.join("2.toml");
        assert!(mapped.is_file());
        assert!(fallback.is_file());

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
