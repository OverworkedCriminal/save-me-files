mod exclusions;
mod files;
mod suffixes;

use anyhow::{anyhow, Result};
use clap::Parser;
use exclusions::read_exclusions;
use files::{find_files_to_copy, calculate_files_size};
use std::path::PathBuf;
use suffixes::read_suffixes;

const COMMENT_LINE_PREFIX: &str = "//";

/// Simple application that finds all files with specified
/// suffixes and copies them to dst_directory.
/// src_directory structure is preserved in dst_directory.
#[derive(Parser)]
struct Args {
    /// Source directory.
    /// Files will be copied starting from this place.
    #[arg(short, long)]
    src_directory: PathBuf,

    /// Destination directory.
    /// All copied files will be copied here.
    #[arg(short, long)]
    dst_directory: PathBuf,

    /// Path to file that stores all suffixes that should be copied
    /// (e.g. '.txt', '.drawio.png' '_backup.txt').
    /// Each suffix should be written in new line.
    #[arg(short, long)]
    include_suffixes_file: PathBuf,

    /// Path to file that stores all excluded paths.
    /// If filepath to copy starts with one of the paths file is ignored.
    /// Paths can be relative to 'src_directory' or absolute.
    /// Each path should be written in new line.
    #[arg(short, long)]
    exclude_paths_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();
    validate_args(&args)?;

    let suffixes = read_suffixes(&args.include_suffixes_file)?;
    let exclusions = args
        .exclude_paths_file
        .map(|path| read_exclusions(path))
        .unwrap_or_else(|| Ok(Vec::new()))?;

    let files_to_copy = find_files_to_copy(&args.src_directory, &suffixes, &exclusions);
    let files_size = calculate_files_size(&files_to_copy);


    Ok(())
}

fn validate_args(
    Args {
        src_directory,
        dst_directory,
        include_suffixes_file,
        exclude_paths_file,
    }: &Args,
) -> Result<()> {
    if !src_directory.is_dir() {
        return Err(anyhow!(
            "src_directory '{}' is not a directory",
            src_directory.to_string_lossy()
        ));
    }

    if !dst_directory.is_dir() {
        return Err(anyhow!(
            "dst_directory '{}' is not a directory",
            dst_directory.to_string_lossy()
        ));
    }

    if !include_suffixes_file.is_file() {
        return Err(anyhow!(
            "include_suffixes_file '{}' is not a file",
            include_suffixes_file.to_string_lossy()
        ));
    }

    if let Some(exclude_paths_file) = exclude_paths_file {
        if !exclude_paths_file.is_file() {
            return Err(anyhow!(
                "exclude_paths_file '{}' is not a file",
                exclude_paths_file.to_string_lossy()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn validate_args_with_exclude() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();
        let exclude_paths_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: Some(exclude_paths_file.path().to_path_buf()),
        };

        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn validate_args_no_exclude() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn validate_args_src_directory_not_exist() {
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: "save-me-files.test.noexistent.file".into(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_src_directory_is_file() {
        let src_directory = NamedTempFile::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_dst_directory_not_exist() {
        let src_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: "save-me-files.test.noexistent.file".into(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_dst_directory_is_file() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = NamedTempFile::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_include_suffixes_file_not_exist() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: "save-me-files.test.noexistent.file".into(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_include_suffixes_file_is_directory() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_exclude_paths_file_not_exist() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: Some("save-me-files.test.noexistent.file".into()),
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_exclude_paths_file_is_directory() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();
        let exclude_paths_file = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: Some(exclude_paths_file.path().to_path_buf()),
        };

        assert!(validate_args(&args).is_err());
    }
}
