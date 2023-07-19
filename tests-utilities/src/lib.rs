use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};
use uuid::Uuid;

pub fn create_unique_tmp_path() -> PathBuf {
    let mut tmp_path = env::temp_dir();
    let mut filename = "save-me-files-".to_string();
    filename.push_str(&Uuid::new_v4().to_string());
    tmp_path.push(filename);
    tmp_path
}

pub fn create_tmp_file() -> PathBuf {
    let tmp_filepath = create_unique_tmp_path();
    File::create(&tmp_filepath).unwrap();
    tmp_filepath
}

pub fn create_tmp_directory() -> PathBuf {
    let tmp_directory_path = create_unique_tmp_path();
    fs::create_dir(&tmp_directory_path).unwrap();
    tmp_directory_path
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_tmp_file_file_exist() {
        let filepath = create_tmp_file();

        let file_exist = filepath.is_file();

        fs::remove_file(filepath).unwrap();
        assert!(file_exist);
    }

    #[test]
    fn create_tmp_directory_directory_exist() {
        let directorypath = create_tmp_directory();

        let directory_exist = directorypath.is_dir();

        fs::remove_dir(directorypath).unwrap();
        assert!(directory_exist);
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
