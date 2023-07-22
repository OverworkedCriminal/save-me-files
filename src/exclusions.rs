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
/// Exclusion is valid when it is path to existing directory.
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
            let dir_exist = path.is_dir();
            if !path.starts_with(COMMENT_LINE_PREFIX) && !dir_exist {
                log::warn!("Exclusion directory not exist: {path:?}");
            }
            dir_exist
        })
        .collect();

    Ok(exclusions)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tests_utilities::{tmp_path, TmpDirectory, TmpFile};

    #[test]
    fn read_exclusions_all_exclusions() {
        let file = TmpFile::new();
        let exclusions = [
            TmpDirectory::new(),
            TmpDirectory::new(),
            TmpDirectory::new(),
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
        let file = TmpFile::new();
        let exclusions = [
            TmpDirectory::new(),
            TmpDirectory::new(),
            TmpDirectory::new(),
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
    fn read_exclusions_ignore_non_existent_directories() {
        let file = TmpFile::new();
        let exclusions = [tmp_path(), tmp_path(), tmp_path()];

        fs::write(
            file.path(),
            format!(
                "{}\n{}\n{}",
                exclusions[0].to_string_lossy(),
                exclusions[1].to_string_lossy(),
                exclusions[2].to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(file.path()).unwrap();

        assert!(read_exclusions.is_empty());
    }
}
