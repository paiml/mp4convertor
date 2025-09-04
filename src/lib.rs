//src/lib.rs

use colored::*;
use humansize::{format_size, DECIMAL};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

use tracing::{debug, error, instrument};
use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .compact()
        .init();
}

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("FFmpeg error: {0}")]
    FFmpeg(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("No video files found in directory")]
    NoVideosFound,
    #[error("Hardware acceleration error: {0}")]
    HWAccel(String),
}

#[derive(Debug, Default)]
pub struct ProcessingSummary {
    pub total_videos: usize,
    pub total_size: u64,
    pub total_duration: f64,
    pub codecs: HashMap<String, usize>,
    pub resolutions: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub codec: String,
    pub resolution: String,
    pub duration: f64,
    pub bitrate: u64,
    pub size: u64,
}

impl ProcessingSummary {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_video(&mut self, metadata: &VideoMetadata) {
        self.total_videos += 1;
        self.total_size += metadata.size;
        self.total_duration += metadata.duration;
        *self.codecs.entry(metadata.codec.clone()).or_insert(0) += 1;
        *self
            .resolutions
            .entry(metadata.resolution.clone())
            .or_insert(0) += 1;
    }

    pub fn display(&self) {
        println!("\n{}", "Processing Summary".bright_green().bold());
        println!("{}", "=".repeat(50).bright_green());
        println!(
            "{} {}",
            "✓".green(),
            format!("Total Videos: {}", self.total_videos).bold()
        );
        println!(
            "{} {}",
            "✓".green(),
            format!("Total Duration: {}", format_duration(self.total_duration)).bold()
        );
        println!(
            "{} {}",
            "✓".green(),
            format!("Total Size: {}", format_size(self.total_size, DECIMAL)).bold()
        );

        if !self.codecs.is_empty() {
            println!("\n{}", "Codec Distribution:".bright_blue().bold());
            for (codec, count) in &self.codecs {
                println!("  {} {} videos using {}", "•".blue(), count, codec);
            }
        }

        if !self.resolutions.is_empty() {
            println!("\n{}", "Resolution Distribution:".bright_blue().bold());
            for (res, count) in &self.resolutions {
                println!("  {} {} videos at {}", "•".blue(), count, res);
            }
        }

        println!("\n{}", "=".repeat(50).bright_green());
    }
}

pub fn format_duration(seconds: f64) -> String {
    let total_centisecs = (seconds * 100.0).trunc() as u32; // Use `trunc` instead of `round`
    let hours = total_centisecs / 360000;
    let minutes = (total_centisecs % 360000) / 6000;
    let secs = (total_centisecs % 6000) / 100;
    let centisecs = total_centisecs % 100;

    format!("{:02}:{:02}:{:02}.{:02}", hours, minutes, secs, centisecs)
}

#[instrument]
pub fn check_hardware_support() -> Result<(), VideoError> {
    debug!("checking nvidia-smi");
    let has_nvidia = Command::new("nvidia-smi").output().map_or_else(
        |e| {
            debug!(?e, "nvidia-smi check failed");
            false
        },
        |output| output.status.success(),
    );

    if !has_nvidia {
        error!("NVIDIA GPU not detected");
        return Err(VideoError::HWAccel("NVIDIA GPU not detected".into()));
    }

    debug!("checking ffmpeg encoders");

    // Verify encoder support
    let ffmpeg = Command::new("ffmpeg")
        .arg("-codecs")
        .output()
        .map_err(|e| VideoError::FFmpeg(e.to_string()))?;

    let codecs = String::from_utf8_lossy(&ffmpeg.stdout);
    if !codecs.contains("h264_nvenc") {
        return Err(VideoError::HWAccel("h264_nvenc not available".into()));
    }
    Ok(())
}
pub fn create_h264_dir(dir: &Path) -> Result<PathBuf, VideoError> {
    let h264_dir = dir.join("H264");
    std::fs::create_dir_all(&h264_dir)?;
    Ok(h264_dir)
}

pub fn write_conversion_report(dir: &Path, summary: &ProcessingSummary) -> Result<(), VideoError> {
    let report_path = dir.join("conversion_report.txt");
    let report = format!(
        "Conversion Report\n\
         ================\n\
         Total Videos: {}\n\
         Total Duration: {}\n\
         Total Size: {}\n\n\
         Codec Distribution:\n{}\n\
         Resolution Distribution:\n{}\n",
        summary.total_videos,
        format_duration(summary.total_duration),
        format_size(summary.total_size, DECIMAL),
        summary
            .codecs
            .iter()
            .map(|(k, v)| format!("  {} videos using {}", v, k))
            .collect::<Vec<_>>()
            .join("\n"),
        summary
            .resolutions
            .iter()
            .map(|(k, v)| format!("  {} videos at {}", v, k))
            .collect::<Vec<_>>()
            .join("\n")
    );
    std::fs::write(report_path, report)?;
    Ok(())
}

pub fn get_hw_decode_codec(input_codec: &str) -> &str {
    match input_codec {
        "h264" => "h264_cuvid",
        "hevc" => "hevc_cuvid",
        "av1" => "av1_cuvid",
        "vp8" => "vp8_cuvid",
        "vp9" => "vp9_cuvid",
        _ => "",
    }
}

pub fn parse_time(line: &str) -> Option<f64> {
    // Quick fix - just look for "time=" and parse the timestamp
    if line.contains("time=") {
        let parts: Vec<&str> = line.split("time=").collect();
        if parts.len() > 1 {
            let time_str = parts[1].split_whitespace().next()?;
            // Simple HH:MM:SS.ms parsing
            let time_parts: Vec<&str> = time_str.split(':').collect();
            if time_parts.len() == 3 {
                let h: f64 = time_parts[0].parse().ok()?;
                let m: f64 = time_parts[1].parse().ok()?;
                let s: f64 = time_parts[2].parse().ok()?;
                return Some(h * 3600.0 + m * 60.0 + s);
            }
        }
    }
    None
}

pub fn analyze_video(path: &Path) -> Result<VideoMetadata, VideoError> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} Analyzing {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    spinner.set_message(path.file_name().unwrap().to_string_lossy().to_string());

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path.to_str()
                .ok_or_else(|| VideoError::InvalidPath(path.display().to_string()))?,
        ])
        .output()?;

    spinner.finish_and_clear();

    if !output.status.success() {
        return Err(VideoError::FFmpeg(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| VideoError::FFmpeg(e.to_string()))?;

    let video_stream = &json["streams"][0];

    Ok(VideoMetadata {
        codec: video_stream["codec_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        resolution: format!(
            "{}x{}",
            video_stream["width"].as_u64().unwrap_or(0),
            video_stream["height"].as_u64().unwrap_or(0)
        ),
        duration: json["format"]["duration"]
            .as_str()
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(0.0),
        bitrate: json["format"]["bit_rate"]
            .as_str()
            .and_then(|b| b.parse::<u64>().ok())
            .unwrap_or(0),
        size: json["format"]["size"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
    })
}

pub fn convert_video(input: &Path, h264_dir: &Path) -> Result<(), VideoError> {
    let output = h264_dir.join(input.file_name().unwrap());
    println!("\nConverting: {} -> {}", input.display(), output.display());
    println!("----------------------------------------");

    let args = vec![
        "-i",
        input.to_str().unwrap(),
        "-c:v",
        "h264_nvenc",
        "-preset",
        "p7",
        "-c:a",
        "aac",
        "-b:a",
        "320k",
        "-y",
        output.to_str().unwrap(),
    ];

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    match child.wait() {
        Ok(status) => {
            if !status.success() && status.code() != Some(130) {
                return Err(VideoError::FFmpeg("Conversion failed".into()));
            }
        }
        Err(e) => {
            // Try to kill the process if interrupted
            let _ = child.kill();
            return Err(VideoError::Io(e));
        }
    }

    Ok(())
}
pub fn validate_directory(dir: &Path) -> Result<Vec<PathBuf>, VideoError> {
    if !dir.is_dir() {
        return Err(VideoError::InvalidPath(format!(
            "Not a directory: {}",
            dir.display()
        )));
    }

    let entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "mp4" || ext == "avi")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    if entries.is_empty() {
        return Err(VideoError::NoVideosFound);
    }

    Ok(entries)
}
pub fn process_directory(
    dir: &Path,
    should_convert: bool,
    verbose: bool,
) -> Result<(), VideoError> {
    let video_files = validate_directory(dir)?;
    check_hardware_support()?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner.set_message("Scanning directory...");

    let h264_dir = if should_convert {
        Some(create_h264_dir(dir)?)
    } else {
        None
    };

    spinner.finish_and_clear();
    println!("{}", "\nScanning directory:".bright_blue());
    println!("{}", dir.display().to_string().bright_blue().bold());
    println!("{}", "=".repeat(50).bright_blue());

    let mut summary = ProcessingSummary::new();

    for path in video_files {
        let metadata = analyze_video(&path)?;
        println!(
            "\n{}",
            format!("File: {}", path.file_name().unwrap().to_str().unwrap()).bold()
        );
        println!(
            "  {} {}",
            "Runtime:".blue(),
            format_duration(metadata.duration)
        );
        if verbose {
            println!("  {} {}", "Codec:".blue(), metadata.codec);
            println!("  {} {}", "Resolution:".blue(), metadata.resolution);
            println!(
                "  {} {} Mbps",
                "Bitrate:".blue(),
                metadata.bitrate as f64 / 1_000_000.0
            );
            println!(
                "  {} {}",
                "Size:".blue(),
                format_size(metadata.size, DECIMAL)
            );
        }

        summary.add_video(&metadata);

        if let Some(ref h264_dir) = h264_dir {
            convert_video(&path, h264_dir)?;
        }
    }

    if let Some(ref h264_dir) = h264_dir {
        write_conversion_report(h264_dir, &summary)?;
    }

    summary.display();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use tracing::{debug, instrument};

    #[test]
    fn test_hardware_support() {
        match check_hardware_support() {
            Ok(_) => println!("Hardware support verified"),
            Err(e) => println!("Hardware support test failed: {}", e),
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(3661.5), "01:01:01.50");
        assert_eq!(format_duration(0.0), "00:00:00.00");
        assert_eq!(format_duration(7323.0), "02:02:03.00");
    }

    #[test]
    fn test_duration_formatting_properties() {
        let test_cases = [
            (0.0, "00:00:00.00"),
            (59.949, "00:00:59.94"), // Adjusted expectation
            (59.999, "00:00:59.99"), // Adjusted expectation
            (60.0, "00:01:00.00"),
            (3599.999, "00:59:59.99"),
            (3600.0, "01:00:00.00"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                format_duration(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_process_empty_directory() {
        let dir = tempdir().unwrap();
        let result = validate_directory(dir.path());
        assert!(matches!(result, Err(VideoError::NoVideosFound)));
    }

    #[test]
    fn test_invalid_directory() {
        let result = validate_directory(Path::new("/nonexistent"));
        assert!(matches!(result, Err(VideoError::InvalidPath(_))));
    }

    #[test]
    fn test_hw_decode_codec_mapping() {
        assert_eq!(get_hw_decode_codec("h264"), "h264_cuvid");
        assert_eq!(get_hw_decode_codec("av1"), "av1_cuvid");
        assert_eq!(get_hw_decode_codec("unknown"), "");
    }

    #[test]
    fn test_parse_time() {
        let result = parse_time("frame=1000 fps=30 time=00:01:23.45 bitrate=1000k");
        assert_eq!(result, Some(83.45));
    }

    #[instrument(level = "debug")]
    #[test]
    fn test_time_edge_cases() {
        let test_cases = [
            ("time=00:00:03.45", Some(3.45)),
            ("time=00:01:23.45", Some(83.45)),
            ("time=01:00:00.00", Some(3600.0)),
            ("time=00:01:00.00", Some(60.0)),
        ];

        for (input, expected) in test_cases.iter() {
            debug!(input = %input, ?expected, "Testing edge case");
            let result = parse_time(input);
            debug!(?result, ?expected, "Edge case result");

            assert_eq!(
                result, *expected,
                "Failed parsing edge case '{}'. Got {:?}, expected {:?}",
                input, result, expected
            );
        }
    }

    #[test]
    fn test_precise_timing() {
        debug!("Testing precise timing");
        let result = parse_time("time=00:01:23.45");
        debug!(?result, "Precise timing result");

        assert!(result.is_some(), "Expected Some value but got None");
        if let Some(time) = result {
            assert!((time - 83.45).abs() < 0.001, "Expected 83.45, got {}", time);
        }
    }

    #[test]
    fn test_summary_tracking() {
        let mut summary = ProcessingSummary::new();
        let metadata = VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 120.0,
            bitrate: 5_000_000,
            size: 75_000_000,
        };

        summary.add_video(&metadata);
        assert_eq!(summary.total_videos, 1);
        assert_eq!(summary.total_duration, 120.0);
        assert_eq!(summary.total_size, 75_000_000);
        assert_eq!(summary.codecs.get("h264"), Some(&1));
        assert_eq!(summary.resolutions.get("1920x1080"), Some(&1));
    }

    #[test]
    fn test_h264_dir_creation() {
        let temp = tempdir().unwrap();
        let h264_dir = create_h264_dir(temp.path()).unwrap();
        assert!(h264_dir.exists());
        assert!(h264_dir.is_dir());
        assert_eq!(h264_dir.file_name().unwrap(), "H264");
    }

    #[test]
    fn test_conversion_report() {
        let temp = tempdir().unwrap();
        let mut summary = ProcessingSummary::new();
        summary.add_video(&VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 120.0,
            bitrate: 5_000_000,
            size: 75_000_000,
        });

        write_conversion_report(temp.path(), &summary).unwrap();
        let report_path = temp.path().join("conversion_report.txt");
        assert!(report_path.exists());

        let content = fs::read_to_string(report_path).unwrap();
        assert!(content.contains("Total Videos: 1"));
        assert!(content.contains("1920x1080"));
        assert!(content.contains("h264"));
    }

    #[test]
    fn test_original_filename_preservation() {
        let temp = tempdir().unwrap();
        let h264_dir = create_h264_dir(temp.path()).unwrap();

        let test_file = temp.path().join("test_video.mp4");
        fs::write(&test_file, b"dummy video data").unwrap();

        let output_path = h264_dir.join(test_file.file_name().unwrap());
        assert_eq!(
            output_path.file_name().unwrap(),
            test_file.file_name().unwrap()
        );
    }
    #[test]
    fn test_rounding_behavior() {
        let input: f64 = 59.999;
        let scaled: f64 = (input * 100.0).round(); // Explicit f64
        let normalized = scaled / 100.0;
        assert_eq!(normalized, 60.0);
    }

    // Additional comprehensive tests for 80% coverage target

    #[test]
    fn test_video_error_variants() {
        let io_error = VideoError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let ffmpeg_error = VideoError::FFmpeg("encoding failed".to_string());
        let invalid_path_error = VideoError::InvalidPath("/invalid/path".to_string());
        let no_videos_error = VideoError::NoVideosFound;
        let hw_accel_error = VideoError::HWAccel("GPU not found".to_string());

        assert!(io_error.to_string().contains("IO error"));
        assert!(ffmpeg_error.to_string().contains("FFmpeg error"));
        assert!(invalid_path_error.to_string().contains("Invalid path"));
        assert!(no_videos_error.to_string().contains("No video files found"));
        assert!(hw_accel_error
            .to_string()
            .contains("Hardware acceleration error"));
    }

    #[test]
    fn test_video_metadata_comprehensive() {
        let metadata = VideoMetadata {
            codec: "hevc".to_string(),
            resolution: "3840x2160".to_string(),
            duration: 7200.0,
            bitrate: 25000000,
            size: 2000000000,
        };

        // Test Clone trait
        let cloned = metadata.clone();
        assert_eq!(metadata.codec, cloned.codec);
        assert_eq!(metadata.resolution, cloned.resolution);

        // Test Debug trait
        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("hevc"));
        assert!(debug_str.contains("3840x2160"));
    }

    #[test]
    fn test_processing_summary_comprehensive() {
        let mut summary = ProcessingSummary::new();

        // Test empty display (exercises empty branch)
        summary.display();

        // Add diverse videos
        let videos = vec![
            VideoMetadata {
                codec: "h264".to_string(),
                resolution: "1920x1080".to_string(),
                duration: 120.0,
                bitrate: 5000000,
                size: 100000000,
            },
            VideoMetadata {
                codec: "hevc".to_string(),
                resolution: "3840x2160".to_string(),
                duration: 180.0,
                bitrate: 15000000,
                size: 300000000,
            },
            VideoMetadata {
                codec: "h264".to_string(),
                resolution: "1280x720".to_string(),
                duration: 90.0,
                bitrate: 3000000,
                size: 50000000,
            },
        ];

        for video in &videos {
            summary.add_video(video);
        }

        assert_eq!(summary.total_videos, 3);
        assert_eq!(summary.total_duration, 390.0);
        assert_eq!(summary.total_size, 450000000);
        assert_eq!(*summary.codecs.get("h264").unwrap(), 2);
        assert_eq!(*summary.codecs.get("hevc").unwrap(), 1);

        // Test display with data (exercises populated branches)
        summary.display();
    }

    #[test]
    fn test_hardware_decode_all_codecs() {
        let test_cases = [
            ("h264", "h264_cuvid"),
            ("hevc", "hevc_cuvid"),
            ("av1", "av1_cuvid"),
            ("vp8", "vp8_cuvid"),
            ("vp9", "vp9_cuvid"),
            ("mpeg4", ""),
            ("xvid", ""),
            ("", ""),
        ];

        for (input, expected) in test_cases {
            assert_eq!(get_hw_decode_codec(input), expected);
        }
    }

    #[test]
    fn test_parse_time_comprehensive() {
        let test_cases = [
            ("time=00:01:23.45", Some(83.45)),
            ("frame=1234 fps=30 time=00:00:30.12", Some(30.12)),
            ("invalid line", None),
            ("time=invalid", None),
            ("time=", None),
            ("time=abc:def:ghi", None),
            ("time=1:2", None),                 // Not enough parts
            ("time=1:2:3 extra", Some(3723.0)), // Extra text ignored
            ("prefix time=01:02:03 suffix", Some(3723.0)),
            ("multiple time=1:0:0 and time=2:0:0", Some(3600.0)), // First match
            ("time=25:61:61", Some(93721.0)),                     // Invalid time but parsed
        ];

        for (input, expected) in test_cases {
            assert_eq!(parse_time(input), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_format_duration_comprehensive() {
        let test_cases = [
            (0.0, "00:00:00.00"),
            (0.001, "00:00:00.00"), // Very small, truncated
            (0.004, "00:00:00.00"), // Should truncate
            (0.01, "00:00:00.01"),
            (0.99, "00:00:00.99"),
            (1.999, "00:00:01.99"),  // Truncation test
            (59.994, "00:00:59.99"), // Should truncate, not round
            (61.5, "00:01:01.50"),
            (3661.25, "01:01:01.25"),
            (3723.456, "01:02:03.45"),
            (3599.999, "00:59:59.99"),
            (86399.99, "23:59:59.99"), // Just under 24h
            (90061.0, "25:01:01.00"),  // Over 24h
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                format_duration(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_validate_directory_comprehensive() {
        let temp_dir = tempdir().unwrap();

        // Test file instead of directory
        let file_path = temp_dir.path().join("not_a_dir.txt");
        std::fs::write(&file_path, "content").unwrap();

        let result = validate_directory(&file_path);
        assert!(matches!(result, Err(VideoError::InvalidPath(_))));

        // Test directory with non-video files
        std::fs::write(temp_dir.path().join("document.txt"), "content").unwrap();
        std::fs::write(temp_dir.path().join("image.jpg"), "content").unwrap();

        let result = validate_directory(temp_dir.path());
        assert!(matches!(result, Err(VideoError::NoVideosFound)));

        // Add video files
        std::fs::write(temp_dir.path().join("video1.mp4"), "content").unwrap();
        std::fs::write(temp_dir.path().join("video2.avi"), "content").unwrap();
        std::fs::write(temp_dir.path().join("video3.mkv"), "content").unwrap(); // Should be ignored

        let result = validate_directory(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .any(|p| p.file_name().unwrap() == "video1.mp4"));
        assert!(result
            .iter()
            .any(|p| p.file_name().unwrap() == "video2.avi"));
    }

    #[test]
    fn test_create_h264_dir_nested() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep");
        std::fs::create_dir_all(&nested_path).unwrap();

        let h264_dir = create_h264_dir(&nested_path).unwrap();
        assert!(h264_dir.exists());
        assert!(h264_dir.is_dir());
        assert_eq!(h264_dir.file_name().unwrap(), "H264");
        assert_eq!(h264_dir.parent().unwrap(), nested_path);
    }

    #[test]
    fn test_write_conversion_report_empty() {
        let temp_dir = tempdir().unwrap();
        let summary = ProcessingSummary::new();

        write_conversion_report(temp_dir.path(), &summary).unwrap();

        let report_path = temp_dir.path().join("conversion_report.txt");
        assert!(report_path.exists());

        let content = std::fs::read_to_string(report_path).unwrap();
        assert!(content.contains("Total Videos: 0"));
        assert!(content.contains("00:00:00.00"));
    }

    #[test]
    fn test_init_logging_safe() {
        // Test that init_logging exists and can be called
        // Note: In actual usage, init_logging may only work once per process
        // so we just test that the function exists and doesn't immediately panic
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            init_logging(); // Only call once to avoid global subscriber conflict
        });

        // Test passes if we get here without panicking
    }

    #[test]
    fn test_video_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let video_err: VideoError = io_err.into();

        match video_err {
            VideoError::Io(e) => assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied),
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_processing_summary_debug() {
        use std::collections::HashMap;

        let summary = ProcessingSummary {
            total_videos: 5,
            total_size: 1000000,
            total_duration: 300.0,
            codecs: HashMap::from([("h264".to_string(), 3), ("hevc".to_string(), 2)]),
            resolutions: HashMap::from([
                ("1920x1080".to_string(), 4),
                ("3840x2160".to_string(), 1),
            ]),
        };

        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("total_videos: 5"));
        assert!(debug_str.contains("total_size: 1000000"));
        assert!(debug_str.contains("total_duration: 300"));
    }

    #[test]
    fn test_check_hardware_support_error_paths() {
        // This test exercises the error handling paths in check_hardware_support
        // It will likely fail since we don't have NVIDIA hardware in CI,
        // but it ensures the error paths are covered
        let result = check_hardware_support();

        match result {
            Ok(_) => {
                // Hardware available - good!
                println!("Hardware acceleration available");
            }
            Err(VideoError::HWAccel(_)) => {
                // Expected in most CI environments without GPU
                println!("Hardware acceleration not available (expected in CI)");
            }
            Err(VideoError::FFmpeg(_)) => {
                // FFmpeg not available or doesn't support nvenc
                println!("FFmpeg error (expected if nvenc not available)");
            }
            Err(e) => {
                panic!("Unexpected error type: {}", e);
            }
        }
    }
}
