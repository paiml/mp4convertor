//src/main.rs

use clap::Parser;
use colored::*;
use mp4converter::google_drive::GoogleDriveClient;
use mp4converter::init_logging;
use mp4converter::{process_directory, ComplianceEngine, VideoError};
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about = "Video Content Standards Compliance System")]
pub struct Args {
    /// Directory containing video files (optional if using Google Drive)
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Convert files (default: only show info)
    #[arg(short, long)]
    pub convert: bool,

    /// Be verbose about operations
    #[arg(short, long)]
    pub verbose: bool,

    /// Analyze content standards compliance
    #[arg(long)]
    pub compliance: bool,

    /// Analyze Google Drive files for compliance
    #[arg(long)]
    pub drive: bool,

    /// Path to Google Drive credentials JSON file
    #[arg(long, default_value = "credentials.json")]
    pub credentials: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), VideoError> {
    init_logging(); // Setup first
    debug!("starting mp4converter");

    let args = Args::parse();
    debug!(?args, "parsed arguments");

    // Validate arguments
    if !args.drive && args.dir.is_none() {
        eprintln!("{} Either --dir must be provided or --drive flag must be set", "Error:".red().bold());
        std::process::exit(1);
    }

    if args.drive {
        // Google Drive mode
        match process_google_drive(&args).await {
            Ok(_) => {
                info!("Google Drive processing completed");
                println!("\n{}", "Google Drive analysis completed successfully!".green().bold());
                Ok(())
            }
            Err(e) => {
                error!(?e, "Google Drive processing failed");
                eprintln!("\n{} {}", "Error:".red().bold(), e);
                Err(e)
            }
        }
    } else if let Some(dir) = &args.dir {
        // Local directory mode  
        match process_directory(dir, args.convert, args.verbose, args.compliance) {
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
    } else {
        unreachable!("Should be caught by argument validation above");
    }
}

async fn process_google_drive(args: &Args) -> Result<(), VideoError> {
    info!("Starting Google Drive compliance analysis");

    // Initialize Google Drive client
    println!("{}", "🔐 Authenticating with Google Drive...".blue().bold());
    let client = GoogleDriveClient::new(&args.credentials).await?;

    // Initialize compliance engine
    println!("{}", "📋 Loading content standards...".blue().bold());
    let compliance_engine = ComplianceEngine::new()?;

    // Perform compliance audit
    println!("{}", "🔍 Scanning Google Drive for video files...".blue().bold());
    
    let report = client.audit_compliance(&compliance_engine, |current, total, filename| {
        println!("📁 Analyzing {}/{}: {}", current, total, filename);
    }).await?;

    // Display results
    report.display();

    Ok(())
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

        assert_eq!(args.dir.as_ref().unwrap().to_str().unwrap(), "/test/path");
        assert!(args.convert);
        assert!(args.verbose);
    }

    #[test]
    fn test_minimal_args() {
        let args = Args::try_parse_from(["mp4converter", "--dir", "/test/path"]).unwrap();

        assert_eq!(args.dir.as_ref().unwrap().to_str().unwrap(), "/test/path");
        assert!(!args.convert); // Default false
        assert!(!args.verbose); // Default false
    }

    #[test]
    fn test_missing_required_args() {
        let result = Args::try_parse_from(["mp4converter"]);
        assert!(result.is_ok()); // Now passes because --dir is not required when --drive can be used
        
        // But dir should be None and drive should be false by default
        let args = result.unwrap();
        assert!(args.dir.is_none());
        assert!(!args.drive);
    }

    #[test]
    fn test_args_debug() {
        let args = Args {
            dir: Some(PathBuf::from("/test/path")),
            convert: true,
            verbose: false,
            compliance: true,
            drive: false,
            credentials: PathBuf::from("credentials.json"),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("/test/path"));
        assert!(debug_str.contains("convert: true"));
        assert!(debug_str.contains("verbose: false"));
        assert!(debug_str.contains("compliance: true"));
        assert!(debug_str.contains("drive: false"));
    }

    #[test]
    fn test_main_function_integration() {
        // Test main function behavior with invalid directory
        let temp_dir = tempdir().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent");

        // Create args manually to test process_directory call
        let result = process_directory(&invalid_path, false, false, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VideoError::InvalidPath(_)));
    }

    #[test]
    fn test_main_function_empty_directory() {
        let temp_dir = tempdir().unwrap();

        let result = process_directory(temp_dir.path(), false, false, false);
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
        let result = process_directory(temp_dir.path(), false, true, false);

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

        assert!(help_str.contains("Video Content Standards Compliance System"));
        assert!(help_str.contains("--dir"));
        assert!(help_str.contains("--convert"));
        assert!(help_str.contains("--verbose"));
        assert!(help_str.contains("--compliance"));
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

        assert_eq!(args.dir.as_ref().unwrap().to_str().unwrap(), "/test/path");
        assert!(args.convert);
        assert!(args.verbose);
    }

    #[test]
    fn test_pathbuf_handling() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let args = Args {
            dir: Some(path.to_path_buf()),
            convert: false,
            verbose: false,
            compliance: false,
            drive: false,
            credentials: PathBuf::from("credentials.json"),
        };

        assert_eq!(args.dir, Some(path.to_path_buf()));
        assert!(path.exists());
        assert!(path.is_dir());
    }
}
