//src/main_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

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
        assert_eq!(parse_time("frame=1000 fps=30 time=00:01:23.45"), Some(83.45));
        assert_eq!(parse_time("invalid input"), None);
        assert_eq!(parse_time("time=00:00:00.00"), Some(0.0));
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
    fn test_video_metadata_parsing() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        let result = analyze_video(path);
        assert!(matches!(result, Err(VideoError::FFmpeg(_))));
    }

    #[test]
    fn test_duration_formatting_properties() {
        let test_cases = [
            (0.0, "00:00:00.00"),
            (59.949, "00:00:59.95"),
            (59.999, "00:00:59.99"),
            (60.0, "00:01:00.00"),
            (3599.999, "00:59:59.99"),
            (3600.0, "01:00:00.00"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(format_duration(input), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let video_error: VideoError = io_error.into();
        assert!(matches!(video_error, VideoError::Io(_)));
    }
}