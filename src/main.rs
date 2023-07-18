use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;

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
    include_extensions_file: PathBuf,

    /// Path to file that stores all excluded paths.
    /// If filepath to copy starts with one of the paths file is ignored.
    /// Paths can be relative to 'src_directory' or absolute.
    /// Each path should be written in new line.
    #[arg(short, long)]
    exclude_paths_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    validate_args(&args)?;

    Ok(())
}

fn validate_args(
    Args {
        src_directory,
        dst_directory,
        include_extensions_file,
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

    if !include_extensions_file.is_file() {
        return Err(anyhow!(
            "include_extensions_file '{}' is not a file",
            include_extensions_file.to_string_lossy()
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
    use std::{
        env,
        fs::{self, File},
        path::PathBuf,
    };
    use uuid::Uuid;

    #[test]
    fn validate_args_with_exclude() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: Some(create_tmp_file()),
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_dir(&args.dst_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();
        fs::remove_file(&args.exclude_paths_file.unwrap()).unwrap();

        assert!(validation_result.is_ok());
    }

    #[test]
    fn validate_args_no_exclude() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_dir(&args.dst_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_ok());
    }

    #[test]
    fn validate_args_src_directory_not_exist() {
        let args = Args {
            src_directory: create_unique_tmp_path(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.dst_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validate_args_src_directory_is_file() {
        let args = Args {
            src_directory: create_tmp_file(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_file(&args.src_directory).unwrap();
        fs::remove_dir(&args.dst_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validate_args_dst_directory_not_exist() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_unique_tmp_path(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validate_args_dst_directory_is_file() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_tmp_file(),
            include_extensions_file: create_tmp_file(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_file(&args.dst_directory).unwrap();
        fs::remove_file(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validate_args_include_extensions_file_not_exist() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_unique_tmp_path(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_dir(&args.dst_directory).unwrap();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validate_args_include_extensions_file_is_directory() {
        let args = Args {
            src_directory: create_tmp_directory(),
            dst_directory: create_tmp_directory(),
            include_extensions_file: create_tmp_directory(),
            exclude_paths_file: None,
        };

        let validation_result = validate_args(&args);

        fs::remove_dir(&args.src_directory).unwrap();
        fs::remove_dir(&args.dst_directory).unwrap();
        fs::remove_dir(&args.include_extensions_file).unwrap();

        assert!(validation_result.is_err());
    }

    fn create_unique_tmp_path() -> PathBuf {
        let mut tmp_path = env::temp_dir();
        let mut filename = "save-me-files-".to_string();
        filename.push_str(&Uuid::new_v4().to_string());
        tmp_path.push(filename);
        tmp_path
    }

    fn create_tmp_file() -> PathBuf {
        let tmp_filepath = create_unique_tmp_path();
        File::create(&tmp_filepath).unwrap();
        tmp_filepath
    }

    fn create_tmp_directory() -> PathBuf {
        let tmp_directory_path = create_unique_tmp_path();
        fs::create_dir(&tmp_directory_path).unwrap();
        tmp_directory_path
    }
}
