use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub fn find_files_to_copy(
    src_directory: &PathBuf,
    suffixes: &Vec<String>,
    exclusions: &Vec<PathBuf>,
) -> Vec<PathBuf> {
    WalkDir::new(src_directory)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| should_keep_entry(entry, &exclusions))
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(err) => {
                log::warn!("{err}");
                None
            }
        })
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| should_copy_file(&entry, suffixes))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn should_keep_entry(entry: &DirEntry, exclusions: &Vec<PathBuf>) -> bool {
    !entry.file_type().is_dir() || !exclusions.iter().any(|path| path == entry.path())
}

fn should_copy_file(entry: &DirEntry, suffixes: &Vec<String>) -> bool {
    let filename = entry.file_name().to_string_lossy();
    suffixes.iter().any(|suffix| filename.ends_with(suffix))
}
