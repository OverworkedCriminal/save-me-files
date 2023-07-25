mod tmp_directoroy;
mod tmp_file;

pub use tmp_directoroy::TmpDirectory;
pub use tmp_file::TmpFile;

use std::{env, path::PathBuf};
use uuid::Uuid;

pub fn tmp_path() -> PathBuf {
    let mut tmp_path = env::temp_dir();
    tmp_path.push(create_unique_filename());
    tmp_path
}

fn create_unique_filename() -> String {
    "save-me-files-test-".to_string() + &Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

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
