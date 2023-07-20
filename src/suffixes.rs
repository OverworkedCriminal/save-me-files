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
    use tests_utilities::create_unique_tmp_path;

    #[test]
    fn read_suffixes_all_suffixes() {
        let filepath = create_unique_tmp_path();
        let suffixes = [".txt", "some.png", "-screenshot-19-05-1948"];

        fs::write(
            &filepath,
            format!("{}\n{}\n{}", suffixes[0], suffixes[1], suffixes[2]),
        )
        .unwrap();

        let read_suffixes = read_suffixes(&filepath).unwrap();

        let contains_all_suffixes = suffixes
            .into_iter()
            .map(|suffix| read_suffixes.contains(&suffix.to_string()))
            .fold(true, |acc, value| acc && value);

        fs::remove_file(filepath).unwrap();

        assert!(contains_all_suffixes);
    }

    #[test]
    fn read_suffixes_trimmed() {
        let filepath = create_unique_tmp_path();
        let suffixes = [".txt", "some.png", "-screenshot-19-05-1948"];

        fs::write(
            &filepath,
            format!(
                "   {}\n{}   \n \t {}  \t",
                suffixes[0], suffixes[1], suffixes[2]
            ),
        )
        .unwrap();

        let read_suffixes = read_suffixes(&filepath).unwrap();

        let contains_all_suffixes = suffixes
            .into_iter()
            .map(|suffix| read_suffixes.contains(&suffix.to_string()))
            .fold(true, |acc, value| acc && value);

        fs::remove_file(filepath).unwrap();

        assert!(contains_all_suffixes);
    }

    #[test]
    fn read_suffixes_ignore_invalid() {
        let filepath = create_unique_tmp_path();
        let suffixes = ["invalid:suffix", "// comment that's also invalid suffix"];

        fs::write(&filepath, format!("{}\n{}\n", suffixes[0], suffixes[1])).unwrap();

        let read_suffixes = read_suffixes(&filepath).unwrap();

        fs::remove_file(filepath).unwrap();

        assert!(read_suffixes.is_empty());
    }
}
