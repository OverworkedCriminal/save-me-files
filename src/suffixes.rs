use anyhow::Result;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const COMMENT_LINE_PREFIX: &str = "//";

pub fn read_suffixes(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let valid_filename_regex = Regex::new(r"^[a-zA-Z0-9_.\-\s]+$").unwrap();

    let suffixes = reader
        .lines()
        .map(|line| line.unwrap().trim().to_owned())
        .filter(|line| {
            let is_valid = valid_filename_regex.is_match(&line);
            if !is_valid && !line.starts_with(COMMENT_LINE_PREFIX) {
                log::warn!("Invalid suffix: {line}");
            }
            is_valid
        })
        .collect();

    Ok(suffixes)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tests_utilities::TmpFile;

    #[test]
    fn read_suffixes_all_suffixes() {
        let file = TmpFile::new();
        let suffixes = [".txt", "some.png", "-screenshot-19-05-1948"];

        fs::write(
            file.path(),
            format!("{}\n{}\n{}", suffixes[0], suffixes[1], suffixes[2]),
        )
        .unwrap();

        let read_suffixes = read_suffixes(file.path()).unwrap();

        suffixes
            .into_iter()
            .for_each(|suffix| assert!(read_suffixes.contains(&suffix.to_string())));
    }

    #[test]
    fn read_suffixes_trimmed() {
        let file = TmpFile::new();
        let suffixes = [".txt", "some.png", "-screenshot-19-05-1948"];

        fs::write(
            file.path(),
            format!(
                "   {}\n{}   \n \t {}  \t",
                suffixes[0], suffixes[1], suffixes[2]
            ),
        )
        .unwrap();

        let read_suffixes = read_suffixes(file.path()).unwrap();

        suffixes
            .into_iter()
            .for_each(|suffix| assert!(read_suffixes.contains(&suffix.to_string())));
    }

    #[test]
    fn read_suffixes_ignore_invalid() {
        let file = TmpFile::new();
        let suffixes = ["invalid:suffix", "// comment that's also invalid suffix"];

        fs::write(file.path(), format!("{}\n{}\n", suffixes[0], suffixes[1])).unwrap();

        let read_suffixes = read_suffixes(file.path()).unwrap();

        assert!(read_suffixes.is_empty());
    }
}
