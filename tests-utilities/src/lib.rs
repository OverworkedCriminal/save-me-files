use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub fn tmp_path() -> PathBuf {
    let mut tmp_path = env::temp_dir();
    tmp_path.push(create_unique_filename());
    tmp_path
}

pub struct TmpFile {
    path: PathBuf,
}

impl TmpFile {
    pub fn new() -> Self {
        let path = tmp_path();
        File::create(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        fs::remove_file(&self.path).unwrap();
    }
}

pub struct TmpDirectory {
    path: PathBuf,
}

impl TmpDirectory {
    pub fn new() -> Self {
        let path = tmp_path();
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for TmpDirectory {
    fn drop(&mut self) {
        fs::remove_dir(&self.path).unwrap();
    }
}

fn create_unique_filename() -> String {
    "save-me-files-test-".to_string() + &Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tmp_file_exist() {
        let tmp_file = TmpFile::new();
        assert!(tmp_file.path().is_file());
    }

    #[test]
    fn tmp_file_is_removed() {
        let tmp_file_path;
        {
            let tmp_file = TmpFile::new();
            tmp_file_path = tmp_file.path().to_path_buf();
            assert!(tmp_file_path.is_file());
        }
        assert!(!tmp_file_path.is_file());
    }

    #[test]
    fn tmp_directory_exist() {
        let tmp_directory = TmpDirectory::new();
        assert!(tmp_directory.path().is_dir());
    }

    #[test]
    fn tmp_directory_is_removed() {
        let tmp_directory_path;
        {
            let tmp_directory = TmpDirectory::new();
            tmp_directory_path = tmp_directory.path().to_path_buf();
            assert!(tmp_directory_path.is_dir());
        }
        assert!(!tmp_directory_path.is_dir());
    }

    #[test]
    fn tmp_path_file_not_exists() {
        let unique_path = tmp_path();
        assert!(!unique_path.exists());
    }

    #[test]
    fn tmp_path_is_unique() {
        let first = tmp_path();
        let second = tmp_path();

        assert_ne!(first, second);
    }

    #[test]
    fn create_unique_filename_name_is_unique() {
        let a = create_unique_filename();
        let b = create_unique_filename();

        assert_ne!(a, b);
    }
}
