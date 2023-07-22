mod exclusions;
mod suffixes;

use anyhow::{anyhow, Result};
use clap::Parser;
use exclusions::read_exclusions;
use std::{env, path::PathBuf};
use suffixes::read_suffixes;

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
    init_logger();

    let args = Args::parse();
    validate_args(&args)?;

    let suffixes = read_suffixes(&args.include_suffixes_file)?;
    let exclusions = args
        .exclude_paths_file
        .map(|path| read_exclusions(path))
        .unwrap_or_else(|| Ok(Vec::new()))?;

    Ok(())
}

fn init_logger() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", log::Level::Info.as_str());
    }
    env_logger::init();
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
    use tests_utilities::{tmp_path, TmpDirectory, TmpFile};

    #[test]
    fn validate_args_with_exclude() {
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();
        let exclude_paths_file = TmpFile::new();

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
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();

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
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();

        let args = Args {
            src_directory: tmp_path(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_src_directory_is_file() {
        let src_directory = TmpFile::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();

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
        let src_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: tmp_path(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_dst_directory_is_file() {
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpFile::new();
        let include_suffixes_file = TmpFile::new();

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
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: tmp_path(),
            exclude_paths_file: None,
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_include_suffixes_file_is_directory() {
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpDirectory::new();

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
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: Some(tmp_path()),
        };

        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn validate_args_exclude_paths_file_is_directory() {
        let src_directory = TmpDirectory::new();
        let dst_directory = TmpDirectory::new();
        let include_suffixes_file = TmpFile::new();
        let exclude_paths_file = TmpDirectory::new();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: include_suffixes_file.path().to_path_buf(),
            exclude_paths_file: Some(exclude_paths_file.path().to_path_buf()),
        };

        assert!(validate_args(&args).is_err());
    }
}
