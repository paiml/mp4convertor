//src/main.rs

use clap::Parser;
use colored::*;
use mp4converter::init_logging;
use mp4converter::{process_directory, VideoError};
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about = "Video processing tool with GPU support")]
pub struct Args {
    /// Directory containing video files
    #[arg(short, long)]
    pub dir: PathBuf,

    /// Convert files (default: only show info)
    #[arg(short, long)]
    pub convert: bool,

    /// Be verbose about operations
    #[arg(short, long)]
    pub verbose: bool,
}

fn main() -> Result<(), VideoError> {
    init_logging(); // Setup first
    debug!("starting mp4converter");

    let args = Args::parse();
    debug!(?args, "parsed arguments");

    match process_directory(&args.dir, args.convert, args.verbose) {
        Ok(_) => {
            info!("processing completed");
            println!("\n{}", "Processing completed successfully!".green().bold());
            Ok(())
        }
        Err(e) => {
            error!(?e, "processing failed");
            eprintln!("\n{} {}", "Error:".red().bold(), e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_args_parsing() {
        // Test that arguments are parsed correctly
        let args = Args::try_parse_from([
            "mp4converter",
            "--dir",
            "/test/path",
            "--convert",
            "--verbose",
        ])
        .unwrap();

        assert_eq!(args.dir.to_str().unwrap(), "/test/path");
        assert!(args.convert);
        assert!(args.verbose);
    }

    #[test]
    fn test_minimal_args() {
        let args = Args::try_parse_from(["mp4converter", "--dir", "/test/path"]).unwrap();

        assert_eq!(args.dir.to_str().unwrap(), "/test/path");
        assert!(!args.convert); // Default false
        assert!(!args.verbose); // Default false
    }

    #[test]
    fn test_missing_required_args() {
        let result = Args::try_parse_from(["mp4converter"]);
        assert!(result.is_err()); // Should fail without --dir
    }

    #[test]
    fn test_args_debug() {
        let args = Args {
            dir: PathBuf::from("/test/path"),
            convert: true,
            verbose: false,
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("/test/path"));
        assert!(debug_str.contains("convert: true"));
        assert!(debug_str.contains("verbose: false"));
    }

    #[test]
    fn test_main_function_integration() {
        // Test main function behavior with invalid directory
        let temp_dir = tempdir().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent");

        // Create args manually to test process_directory call
        let result = process_directory(&invalid_path, false, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VideoError::InvalidPath(_)));
    }

    #[test]
    fn test_main_function_empty_directory() {
        let temp_dir = tempdir().unwrap();

        let result = process_directory(temp_dir.path(), false, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VideoError::NoVideosFound));
    }

    #[test]
    fn test_main_function_with_dummy_videos() {
        let temp_dir = tempdir().unwrap();

        // Create dummy video files
        fs::write(temp_dir.path().join("test1.mp4"), "dummy content").unwrap();
        fs::write(temp_dir.path().join("test2.avi"), "dummy content").unwrap();

        // This will likely fail due to ffprobe requirements, but tests the path
        let result = process_directory(temp_dir.path(), false, true);

        // Should either succeed (if ffprobe available) or fail with FFmpeg error
        match result {
            Ok(_) => {
                // Success - ffprobe worked
                println!("Integration test succeeded with real ffprobe");
            }
            Err(VideoError::FFmpeg(_)) => {
                // Expected - ffprobe likely not available or failed on dummy files
                println!("FFmpeg error as expected with dummy files");
            }
            Err(e) => {
                panic!("Unexpected error type: {}", e);
            }
        }
    }

    #[test]
    fn test_clap_help_text() {
        use clap::CommandFactory;

        let mut cmd = Args::command();
        let help = cmd.render_help();
        let help_str = help.to_string();

        assert!(help_str.contains("Video processing tool"));
        assert!(help_str.contains("--dir"));
        assert!(help_str.contains("--convert"));
        assert!(help_str.contains("--verbose"));
        assert!(help_str.contains("Directory containing video files"));
    }

    #[test]
    fn test_clap_version() {
        use clap::CommandFactory;

        let cmd = Args::command();
        let version = cmd.get_version();

        // Should have a version (from Cargo.toml)
        assert!(version.is_some());
    }

    #[test]
    fn test_args_short_flags() {
        let args = Args::try_parse_from(["mp4converter", "-d", "/test/path", "-c", "-v"]).unwrap();

        assert_eq!(args.dir.to_str().unwrap(), "/test/path");
        assert!(args.convert);
        assert!(args.verbose);
    }

    #[test]
    fn test_pathbuf_handling() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let args = Args {
            dir: path.to_path_buf(),
            convert: false,
            verbose: false,
        };

        assert_eq!(args.dir, path);
        assert!(args.dir.exists());
        assert!(args.dir.is_dir());
    }
}
