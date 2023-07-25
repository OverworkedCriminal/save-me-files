use crate::tmp_path;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_file_exist() {
        let tmp_file = TmpFile::new();
        assert!(tmp_file.path().is_file());
    }

    #[test]
    fn drop_file_is_removed() {
        let tmp_file_path;
        {
            let tmp_file = TmpFile::new();
            tmp_file_path = tmp_file.path().to_path_buf();
            assert!(tmp_file_path.is_file());
        }
        assert!(!tmp_file_path.is_file());
    }
}
