use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const COMMENT_LINE_PREFIX: &str = "//";

pub fn read_exclusions(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let exclusions = reader
        .lines()
        .into_iter()
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
    use tests_utilities::{create_tmp_directory, create_tmp_file, create_unique_tmp_path};

    #[test]
    fn read_exclusions_all_exclusions() {
        let filepath = create_tmp_file();
        let exclusions = [
            create_tmp_directory(),
            create_tmp_directory(),
            create_tmp_directory(),
        ];

        fs::write(
            &filepath,
            format!(
                "{}\n{}\n{}",
                exclusions[0].to_string_lossy(),
                exclusions[1].to_string_lossy(),
                exclusions[2].to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(filepath).unwrap();

        let contains_all_exclusions = exclusions
            .iter()
            .map(|exclusion| read_exclusions.contains(exclusion))
            .fold(true, |acc, value| acc && value);
        exclusions
            .into_iter()
            .for_each(|path| fs::remove_dir(path).unwrap());

        assert!(contains_all_exclusions);
    }

    #[test]
    fn read_exclusions_all_exclusions_trimmed() {
        let filepath = create_tmp_file();
        let exclusions = [
            create_tmp_directory(),
            create_tmp_directory(),
            create_tmp_directory(),
        ];

        fs::write(
            &filepath,
            format!(
                " \t {}\n{} \t \n \t {} \t ",
                exclusions[0].to_string_lossy(),
                exclusions[1].to_string_lossy(),
                exclusions[2].to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(filepath).unwrap();

        let contains_all_exclusions = exclusions
            .iter()
            .map(|exclusion| read_exclusions.contains(exclusion))
            .fold(true, |acc, value| acc && value);
        exclusions
            .into_iter()
            .for_each(|path| fs::remove_dir(path).unwrap());

        assert!(contains_all_exclusions);
    }

    #[test]
    fn read_exclusions_ignore_non_existent_directories() {
        let filepath = create_tmp_file();
        let exclusions = [
            create_unique_tmp_path(),
            create_unique_tmp_path(),
            create_unique_tmp_path(),
        ];

        fs::write(
            &filepath,
            format!(
                "{}\n{}\n{}",
                exclusions[0].to_string_lossy(),
                exclusions[1].to_string_lossy(),
                exclusions[2].to_string_lossy()
            ),
        )
        .unwrap();

        let read_exclusions = read_exclusions(filepath).unwrap();

        assert!(read_exclusions.is_empty());
    }
}
