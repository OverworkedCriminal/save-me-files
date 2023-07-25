use crate::{create_unique_filename, tmp_path};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct TmpDirectory {
    path: PathBuf,
}

impl TmpDirectory {
    pub fn new() -> Self {
        let path = tmp_path();
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    pub fn new_with_parent(parent: &Path) -> Self {
        let path = parent.join(create_unique_filename());
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
    fn new_directory_exist() {
        let tmp_directory = TmpDirectory::new();
        assert!(tmp_directory.path().is_dir());
    }

    #[test]
    fn drop_directory_is_removed() {
        let tmp_directory_path;
        {
            let tmp_directory = TmpDirectory::new();
            tmp_directory_path = tmp_directory.path().to_path_buf();
            assert!(tmp_directory_path.is_dir());
        }
        assert!(!tmp_directory_path.is_dir());
    }

    #[test]
    fn new_with_parent_directory_exist() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let directory_exist;
        {
            let child = TmpDirectory::new_with_parent(&parent_path);
            directory_exist = child.path().is_dir();
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(directory_exist);
    }

    #[test]
    fn new_with_parent_correct_parent() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let is_parent_correct;
        {
            let child = TmpDirectory::new_with_parent(&parent_path);
            is_parent_correct = child.path().parent().unwrap() == parent_path;
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(is_parent_correct);
    }
}
