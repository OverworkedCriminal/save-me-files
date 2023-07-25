use crate::{create_unique_filename, tmp_path};
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

    pub fn new_with_parent(parent: &Path) -> Self {
        let path = parent.join(create_unique_filename());
        File::create(&path).unwrap();
        Self { path }
    }

    pub fn new_with_parent_and_name(parent: &Path, name: &str) -> Self {
        let path = parent.join(name);
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

    #[test]
    fn new_with_parent_directory_exist() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let file_exist;
        {
            let child = TmpFile::new_with_parent(&parent_path);
            file_exist = child.path().is_file();
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(file_exist);
    }

    #[test]
    fn new_with_parent_correct_parent() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let is_parent_correct;
        {
            let child = TmpFile::new_with_parent(&parent_path);
            is_parent_correct = child.path().parent().unwrap() == parent_path;
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(is_parent_correct);
    }

    #[test]
    fn new_with_parent_and_name_file_exist() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let file_exits;
        {
            let child = TmpFile::new_with_parent_and_name(&parent_path, "valid name.txt");
            file_exits = child.path().is_file();
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(file_exits);
    }

    #[test]
    fn new_with_parent_and_name_correct_parent() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();

        let is_parent_correct;
        {
            let child = TmpFile::new_with_parent_and_name(&parent_path, "valid name.txt");
            is_parent_correct = child.path().parent().unwrap() == parent_path;
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(is_parent_correct);
    }

    #[test]
    fn new_with_parent_and_name_correct_name() {
        let parent_path = tmp_path();
        fs::create_dir(&parent_path).unwrap();
        let name = "filename.txt";

        let is_name_correct;
        {
            let child = TmpFile::new_with_parent_and_name(&parent_path, name);
            is_name_correct = child.path().file_name().unwrap().to_string_lossy() == name;
        }

        fs::remove_dir(&parent_path).unwrap();
        assert!(is_name_correct);
    }
}
