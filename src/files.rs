use byte_unit::Byte;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

///
/// Find files that should be copied and return their paths.
///
/// Files to copy need to end with one of the suffixes. Furthermore
/// File's path can not start with any of the exclusions
///
pub fn find_files_to_copy(
    src_directory: &Path,
    suffixes: &[String],
    exclusions: &[PathBuf],
) -> Vec<PathBuf> {
    WalkDir::new(src_directory)
        .min_depth(0)
        .into_iter()
        .filter_entry(|entry| should_keep_entry(entry, exclusions))
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

///
/// Calculate sum of files sizes.
///
/// When there's problem with reading file metadata, error is logged
/// and file size is ignored.
///
pub fn calculate_files_size(files_paths: &Vec<PathBuf>) -> u64 {
    files_paths
        .iter()
        .map(|file_path| std::fs::metadata(file_path))
        .filter_map(|metadata| match metadata {
            Ok(metadata) => Some(metadata.len()),
            Err(err) => {
                log::warn!("{err}");
                None
            }
        })
        .sum::<u64>()
}

///
/// Copy files at paths by replacing src_directory
/// prefix with dst_directory.
///
pub fn copy_files(src_directory: &Path, dst_directory: &Path, paths: &[PathBuf]) {
    paths
        .into_par_iter()
        .map(|path| {
            let stripped = path.strip_prefix(src_directory).unwrap();
            let src_path = path;
            let dst_path = dst_directory.join(stripped);

            (src_path, dst_path)
        })
        .for_each(|(src_path, dst_path)| {
            create_directories(dst_directory, &dst_path);
            copy_file(src_path, &dst_path);
        });
}

fn should_keep_entry(entry: &DirEntry, exclusions: &[PathBuf]) -> bool {
    !entry.file_type().is_dir() || !exclusions.iter().any(|path| entry.path().starts_with(path))
}

fn should_copy_file(entry: &DirEntry, suffixes: &[String]) -> bool {
    let filename = entry.file_name().to_string_lossy();
    suffixes.iter().any(|suffix| filename.ends_with(suffix))
}

fn create_directories(dst_root: &Path, dst: &Path) {
    let mut dir_path = dst_root.to_path_buf();
    let dst_components = dst
        .parent()
        .unwrap()
        .strip_prefix(dst_root)
        .unwrap()
        .components();
    for component in dst_components {
        dir_path.push(component);
        if dir_path.is_dir() {
            continue;
        }

        if let Err(err) = fs::create_dir(&dir_path) {
            log::warn!(
                "Failed to create parent directories for {}; {err}",
                dst.to_string_lossy()
            );
            break;
        }
    }
}

fn copy_file(src: &Path, dst: &Path) {
    match fs::copy(src, dst) {
        Ok(bytes_copied) => {
            let bytes = Byte::from_bytes(bytes_copied as u128).get_appropriate_unit(true);
            log::info!(
                "Copied {} from {} to {}",
                bytes,
                src.to_string_lossy(),
                dst.to_string_lossy()
            )
        }
        Err(err) => log::warn!("{err}"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
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
            .collect::<Vec<_>>();
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
            .collect::<Vec<_>>();
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
            .collect::<Vec<_>>();
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
            .collect::<Vec<_>>();
        let exclusions = [&dirs[1], &dirs[3]]
            .iter()
            .map(|dir| dir.path().to_path_buf())
            .collect::<Vec<_>>();

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert_eq!(found_files.len(), remaining_files.len());

        remaining_files
            .iter()
            .for_each(|file| assert!(found_files.contains(&file.path().to_path_buf())))
    }

    #[test]
    fn find_files_to_copy_exclude_path_above_src() {
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
            .collect::<Vec<_>>();
        let exclusions = vec![dirs[0].path().parent().unwrap().to_path_buf()];

        let found_files = find_files_to_copy(&root_dir, &suffixes, &exclusions);
        assert!(found_files.is_empty());
    }

    #[test]
    fn calculate_files_size_correct_sum() {
        let files = [
            NamedTempFile::new().unwrap(),
            NamedTempFile::new().unwrap(),
            NamedTempFile::new().unwrap(),
        ];
        fs::write(&files[0], "some unimportant text").unwrap();
        fs::write(&files[1], "event more text").unwrap();
        fs::write(&files[2], "That's the last one").unwrap();
        let files_paths = files
            .iter()
            .map(|file| file.path().to_path_buf())
            .collect::<Vec<_>>();
        let files_size = files
            .iter()
            .map(|file| fs::metadata(&file).unwrap().len())
            .sum::<u64>();

        let calculated_size = calculate_files_size(&files_paths);

        assert_eq!(files_size, calculated_size);
    }

    #[test]
    fn copy_files_contents_are_preserved() {
        let (dirs, files) = create_temp_dir_tree();
        let src_dir = &dirs[1];
        let dst_dir = TempDir::new().unwrap();
        let src_paths = files[2..=3]
            .into_iter()
            .map(|file| file.path().to_path_buf())
            .collect::<Vec<_>>();
        let p1_src_text = "some boring text";
        let p2_src_text = "another boring text";
        fs::write(&src_paths[0], p1_src_text).unwrap();
        fs::write(&src_paths[1], p2_src_text).unwrap();

        copy_files(src_dir.path(), dst_dir.path(), &src_paths);

        let dst_paths = src_paths
            .iter()
            .map(|path| dst_dir.path().join(path.strip_prefix(src_dir).unwrap()))
            .collect::<Vec<_>>();
        assert!(dst_paths[0].is_file());
        assert!(dst_paths[1].is_file());
        let p1_dst_text = fs::read_to_string(&dst_paths[0]).unwrap();
        let p2_dst_text = fs::read_to_string(&dst_paths[1]).unwrap();
        assert_eq!(p1_dst_text, p1_src_text);
        assert_eq!(p2_dst_text, p2_src_text);
    }

    #[test]
    fn copy_files_directory_structure_preserved() {
        let (dirs, files) = create_temp_dir_tree();
        let src_dir = &dirs[2];
        let dst_dir = TempDir::new().unwrap();
        let paths = [files[4].path().to_path_buf()];

        copy_files(src_dir.path(), dst_dir.path(), &paths);

        let td3_stripped = dirs[3].path().strip_prefix(src_dir).unwrap();
        let td3_dst = dst_dir.path().join(td3_stripped);
        let ntf4_stripped = files[4].path().strip_prefix(src_dir).unwrap();
        let ntf4_dst = dst_dir.path().join(ntf4_stripped);

        assert!(td3_dst.is_dir());
        assert!(ntf4_dst.is_file());
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
