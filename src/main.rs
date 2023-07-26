mod exclusions;
mod files;
mod suffixes;

use anyhow::{anyhow, Result};
use byte_unit::Byte;
use clap::Parser;
use exclusions::read_exclusions;
use files::{calculate_files_size, copy_files, find_files_to_copy};
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
    include_suffixes_file: Option<PathBuf>,

    /// Path to file that stores all excluded paths.
    /// If filepath to copy starts with one of the paths file is ignored.
    /// Paths can be relative to 'src_directory' or absolute.
    /// Each path should be written in new line.
    #[arg(short, long)]
    exclude_paths_file: Option<PathBuf>,

    /// Disable copying.
    /// If present makes sure application stops before copying files.
    /// It's useful when someone wants to check what files will be copied.
    #[arg(long, default_value_t = false)]
    no_copy: bool,
}

fn main() -> Result<()> {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut args = Args::parse();
    args = canonicalize_args(args)?;

    let suffixes = args
        .include_suffixes_file
        .map(|path| {
            log::info!("Reading suffixes from {}", path.to_string_lossy());
            read_suffixes(path)
        })
        .unwrap_or_else(|| Ok(vec!["".to_string()]))?;
    let exclusions = args
        .exclude_paths_file
        .map(|path| {
            log::info!("Reading exclusions from {}", path.to_string_lossy());
            read_exclusions(path)
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;

    log::info!(
        "Searching for files to copy starting at {}",
        args.src_directory.to_string_lossy()
    );
    let files_to_copy = find_files_to_copy(&args.src_directory, &suffixes, &exclusions);
    for file_path in files_to_copy.iter() {
        log::info!("Will copy: {}", file_path.to_string_lossy());
    }

    let needed_space = calculate_files_size(&files_to_copy);
    let available_space = fs4::available_space(&args.dst_directory).expect(&format!(
        "Failed to read available space at {}",
        args.dst_directory.to_string_lossy()
    ));
    if needed_space > available_space {
        let needed_space = Byte::from_bytes(needed_space as u128).get_appropriate_unit(true);
        let available_space = Byte::from_bytes(available_space as u128).get_appropriate_unit(true);
        return Err(anyhow!(
            "There's not enough space to copy all files! Needed space {}, available space {}",
            needed_space,
            available_space
        ));
    }

    if args.no_copy {
        log::info!("Copying skipped");
        return Ok(());
    }

    log::info!("Copying files");
    copy_files(&args.src_directory, &args.dst_directory, &files_to_copy);

    Ok(())
}

fn canonicalize_args(mut args: Args) -> Result<Args> {
    if !args.src_directory.is_dir() {
        return Err(anyhow!(
            "src_directory '{}' is not a directory",
            args.src_directory.to_string_lossy()
        ));
    }

    if !args.dst_directory.is_dir() {
        return Err(anyhow!(
            "dst_directory '{}' is not a directory",
            args.dst_directory.to_string_lossy()
        ));
    }

    if let Some(include_suffixes_file) = &args.include_suffixes_file {
        if !include_suffixes_file.is_file() {
            return Err(anyhow!(
                "include_suffixes_file '{}' is not a file",
                include_suffixes_file.to_string_lossy()
            ));
        }
        args.include_suffixes_file = Some(include_suffixes_file.canonicalize().unwrap());
    }

    if let Some(exclude_paths_file) = &args.exclude_paths_file {
        if !exclude_paths_file.is_file() {
            return Err(anyhow!(
                "exclude_paths_file '{}' is not a file",
                exclude_paths_file.to_string_lossy()
            ));
        }
        args.exclude_paths_file = Some(exclude_paths_file.canonicalize().unwrap());
    }

    args.src_directory = args.src_directory.canonicalize().unwrap();
    args.dst_directory = args.dst_directory.canonicalize().unwrap();

    Ok(args)
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn canonicalize_args_all_args_present() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();
        let exclude_paths_file = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: Some(include_suffixes_file.path().to_path_buf()),
            exclude_paths_file: Some(exclude_paths_file.path().to_path_buf()),
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_ok());
    }

    #[test]
    fn canonicalize_args_no_optional_args() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_ok());
    }

    #[test]
    fn canonicalize_args_src_directory_not_exist() {
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: "save-me-files.test.noexistent.file".into(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_src_directory_is_file() {
        let src_directory = NamedTempFile::new().unwrap();
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_dst_directory_not_exist() {
        let src_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: "save-me-files.test.noexistent.file".into(),
            include_suffixes_file: None,
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_dst_directory_is_file() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = NamedTempFile::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_include_suffixes_file_not_exist() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: Some("save-me-files.test.noexistent.file".into()),
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_include_suffixes_file_is_directory() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: Some(include_suffixes_file.path().to_path_buf()),
            exclude_paths_file: None,
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_exclude_paths_file_not_exist() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: Some("save-me-files.test.noexistent.file".into()),
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_exclude_paths_file_is_directory() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let exclude_paths_file = TempDir::new().unwrap();

        let args = Args {
            src_directory: src_directory.path().to_path_buf(),
            dst_directory: dst_directory.path().to_path_buf(),
            include_suffixes_file: None,
            exclude_paths_file: Some(exclude_paths_file.path().to_path_buf()),
            no_copy: false,
        };

        assert!(canonicalize_args(args).is_err());
    }

    #[test]
    fn canonicalize_args_paths_are_absolute() {
        let src_directory = TempDir::new().unwrap();
        let dst_directory = TempDir::new().unwrap();
        let include_suffixes_file = NamedTempFile::new().unwrap();
        let exclude_paths_file = NamedTempFile::new().unwrap();

        let root = std::env::temp_dir();
        std::env::set_current_dir(&root).unwrap();

        let mut args = Args {
            src_directory: src_directory
                .path()
                .strip_prefix(&root)
                .unwrap()
                .to_path_buf(),
            dst_directory: dst_directory
                .path()
                .strip_prefix(&root)
                .unwrap()
                .to_path_buf(),
            include_suffixes_file: Some(
                include_suffixes_file
                    .path()
                    .strip_prefix(&root)
                    .unwrap()
                    .to_path_buf(),
            ),
            exclude_paths_file: Some(
                exclude_paths_file
                    .path()
                    .strip_prefix(&root)
                    .unwrap()
                    .to_path_buf(),
            ),
            no_copy: false,
        };

        args = canonicalize_args(args).unwrap();

        assert!(args.src_directory.is_absolute());
        assert!(args.dst_directory.is_absolute());
        assert!(args.include_suffixes_file.unwrap().is_absolute());
        assert!(args.exclude_paths_file.unwrap().is_absolute());
    }
}
