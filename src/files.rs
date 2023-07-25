use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn find_files_to_copy(
    src_directory: &Path,
    suffixes: &Vec<String>,
    exclusions: &Vec<PathBuf>,
) -> Vec<PathBuf> {
    WalkDir::new(src_directory)
        .min_depth(0)
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

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn find_files_to_copy_all_files() {
        let (dirs, files) = create_temp_dir_tree();
        let root_dir = dirs[0].path();
        let suffixes = files
            .iter()
            .map(|file| {
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        let exclusions = Vec::new();

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert_eq!(found_files.len(), files.len());

        files
            .iter()
            .for_each(|file| assert!(found_files.contains(&file.path().to_path_buf())));
    }

    #[test]
    fn find_files_to_copy_some_files() {
        let (dirs, files) = create_temp_dir_tree();
        let root_dir = dirs[0].path();
        let some_files = [&files[2], &files[4]];
        let suffixes = some_files
            .iter()
            .map(|file| {
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        let exclusions = Vec::new();

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert_eq!(found_files.len(), some_files.len());

        some_files
            .iter()
            .for_each(|file| assert!(found_files.contains(&file.path().to_path_buf())));
    }

    #[test]
    fn find_files_to_copy_exclude_all() {
        let (dirs, files) = create_temp_dir_tree();
        let root_dir = dirs[0].path();
        let suffixes = files
            .iter()
            .map(|file| {
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        let exclusions = vec![dirs[0].path().to_path_buf()];

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert!(found_files.is_empty());
    }

    #[test]
    fn find_files_to_copy_exclude_some() {
        let (dirs, files) = create_temp_dir_tree();
        let root_dir = dirs[0].path();
        let remaining_files = [&files[0], &files[1]];
        let suffixes = files
            .iter()
            .map(|file| {
                file.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        let exclusions = [&dirs[1], &dirs[3]]
            .iter()
            .map(|dir| dir.path().to_path_buf())
            .collect();

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert_eq!(found_files.len(), remaining_files.len());

        remaining_files
            .iter()
            .for_each(|file| assert!(found_files.contains(&file.path().to_path_buf())))
    }

    ///
    /// Creates directory tree.
    /// Returns tuple with arrays of directories and files.
    ///
    /// ```not-rust
    /// TD: 0
    ///  |- NTF: 0
    ///  |- NTF: 1
    ///  |- TD: 1
    ///      |- NTF: 2
    ///      |- NTF: 3
    ///  |- TD: 2
    ///      |- TD: 3
    ///          |- NTF: 4
    /// ```
    fn create_temp_dir_tree() -> ([TempDir; 4], [NamedTempFile; 5]) {
        let td0 = TempDir::new().unwrap();
        let ntf0 = NamedTempFile::new_in(td0.path()).unwrap();
        let ntf1 = NamedTempFile::new_in(td0.path()).unwrap();

        let td1 = TempDir::new_in(td0.path()).unwrap();
        let ntf2 = NamedTempFile::new_in(td1.path()).unwrap();
        let ntf3 = NamedTempFile::new_in(td1.path()).unwrap();

        let td2 = TempDir::new_in(td0.path()).unwrap();
        let td3 = TempDir::new_in(td2.path()).unwrap();
        let ntf4 = NamedTempFile::new_in(td3.path()).unwrap();

        ([td0, td1, td2, td3], [ntf0, ntf1, ntf2, ntf3, ntf4])
    }
}
