use std::{
    env,
    fs::{self, File},
    path::{PathBuf, Path},
};
use uuid::Uuid;

pub fn create_unique_tmp_path() -> PathBuf {
    let mut tmp_path = env::temp_dir();
    let mut filename = "save-me-files-".to_string();
    filename.push_str(&Uuid::new_v4().to_string());
    tmp_path.push(filename);
    tmp_path
}

pub struct TmpFile {
    path: PathBuf,
}

impl TmpFile {
    pub fn new() -> Self {
        let path = create_unique_tmp_path();
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
        let path = create_unique_tmp_path();
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
    fn create_unique_tmp_path_file_not_exists() {
        let unique_path = create_unique_tmp_path();
        assert!(!unique_path.exists());
    }

    #[test]
    fn create_unique_tmp_path_uniqueness() {
        let first = create_unique_tmp_path();
        let second = create_unique_tmp_path();

        assert_ne!(first, second);
    }
}
