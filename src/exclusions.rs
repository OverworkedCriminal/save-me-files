use crate::COMMENT_LINE_PREFIX;
use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

///
/// Read exclusions from file to the vector.
///
/// Exclusions are trimmed so they don't contain leading and following
/// whitespaces.
/// Exclusion is valid when it is an absolute path to existing directory.
/// Every invalid exclusion is logged with WARN level unless
/// it starts with [COMMENT_LINE_PREFIX].
///
/// #### Errors
/// This function returns error when there's a problem with
/// opening the file.
///
/// #### Panics
/// This function panics when input file contains not valid
/// UTF-8 characters.
///
pub fn read_exclusions(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let exclusions = reader
        .lines()
        .map(|line| PathBuf::from(line.unwrap().trim()))
        .filter(|path| {
            if !path.is_absolute() {
                log::warn!(
                    "Exclusion directory is not an absolute path: {}",
                    path.to_string_lossy()
                );
                return false;
            }
            if path.to_string_lossy().starts_with(COMMENT_LINE_PREFIX) {
                return false;
            }
            if !path.is_dir() {
                log::warn!("Exclusion directory not exist: {}", path.to_string_lossy());
                return false;
            }

            return true;
        })
        .collect();

    Ok(exclusions)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn read_exclusions_all_exclusions() {
        let file = NamedTempFile::new().unwrap();
        let exclusions = [
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
        ];

        fs::write(
            file.path(),
            format!(
                "{}\n{}\n{}",
                exclusions[0].path().to_string_lossy(),
                exclusions[1].path().to_string_lossy(),
                exclusions[2].path().to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        exclusions
            .into_iter()
            .map(|exclusion_directory| exclusion_directory.path().to_path_buf())
            .for_each(|path| assert!(read_exclusions.contains(&path)));
    }

    #[test]
    fn read_exclusions_all_exclusions_trimmed() {
        let file = NamedTempFile::new().unwrap();
        let exclusions = [
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
            TempDir::new().unwrap(),
        ];

        fs::write(
            file.path(),
            format!(
                " \t {}\n{} \t \n \t {} \t ",
                exclusions[0].path().to_string_lossy(),
                exclusions[1].path().to_string_lossy(),
                exclusions[2].path().to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        exclusions
            .into_iter()
            .map(|exclusion_directory| exclusion_directory.path().to_path_buf())
            .for_each(|path| assert!(read_exclusions.contains(&path)));
    }

    #[test]
    fn read_exclusions_ignore_comments() {
        let file = NamedTempFile::new().unwrap();

        fs::write(
            file.path(),
            format!(
                "{} {}",
                COMMENT_LINE_PREFIX, "save-me-files.test.noexistent.file"
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        assert!(read_exclusions.is_empty());
    }

    #[test]
    fn read_exclusions_ignore_non_existent_directories() {
        let file = NamedTempFile::new().unwrap();
        let exclusions = [
            "save-me-files.test.noexistent.file1",
            "save-me-files.test.noexistent.file2",
            "save-me-files.test.noexistent.file3",
        ];

        fs::write(
            file.path(),
            format!("{}\n{}\n{}", exclusions[0], exclusions[1], exclusions[2]),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        assert!(read_exclusions.is_empty());
    }

    #[test]
    fn read_exclusions_ignore_relative_paths() {
        let file = NamedTempFile::new().unwrap();
        let exclusion = TempDir::new().unwrap();

        fs::write(
            file.path(),
            format!(
                "{}",
                exclusion.path().file_name().unwrap().to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        assert!(read_exclusions.is_empty());
    }
}
