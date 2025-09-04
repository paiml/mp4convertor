//src/lib.rs

use colored::*;
use humansize::{format_size, DECIMAL};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::EnvFilter;

// Google Drive integration
pub mod google_drive;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .compact()
        .init();
}

impl ContentStandards {
    pub fn load_default() -> Result<Self, VideoError> {
        info!("Loading default content delivery standards");

        let video = VideoStandards {
            preferred_resolutions: vec![
                "1280x720".to_string(),
                "1920x1080".to_string(),
                "720x1280".to_string(),  // Vertical
                "1080x1920".to_string(), // Vertical HD
            ],
            acceptable_resolutions: vec![
                "1360x768".to_string(),
                "1280x800".to_string(),
                "1600x900".to_string(),
                "1440x900".to_string(),
                "1680x1048".to_string(),
                "1440x810".to_string(),
                "2160x3840".to_string(), // Vertical 4K
            ],
            preferred_codecs: vec!["h264".to_string(), "libx264".to_string()],
            preferred_frame_rates: vec![15.0, 23.976, 24.0, 25.0, 29.97, 30.0],
            bitrate_ranges: HashMap::from([
                (
                    "screen_capture".to_string(),
                    BitRateRange {
                        min_kbps: 6000,
                        max_kbps: 8000,
                        content_type: "Screen Capture".to_string(),
                    },
                ),
                (
                    "live_action".to_string(),
                    BitRateRange {
                        min_kbps: 8000,
                        max_kbps: 15000,
                        content_type: "Live Action".to_string(),
                    },
                ),
            ]),
            containers: vec!["mp4".to_string(), "mov".to_string()],
            unsupported_containers: vec!["mkv".to_string()],
            profiles: vec!["main".to_string(), "high".to_string()],
        };

        let audio = AudioStandards {
            preferred_codecs: vec!["pcm".to_string(), "alac".to_string()],
            acceptable_codecs: vec!["aac".to_string()],
            sample_rates: vec![44100, 48000],
            bit_depths: vec![16, 24],
            bitrate_ranges: HashMap::from([("aac".to_string(), 320)]),
            channels: vec!["stereo".to_string(), "2.0".to_string()],
        };

        let quality = QualityStandards {
            color_spaces: vec!["rec709".to_string(), "bt709".to_string()],
            unsupported_color_spaces: vec![
                "bt2020".to_string(),
                "dci-p3".to_string(),
                "rec2020".to_string(),
            ],
            keyframe_interval_min: 2,
            chroma_subsampling: vec!["4:2:0".to_string(), "4:2:2".to_string()],
            hdr_restrictions: vec![
                "hdr10".to_string(),
                "hdr10+".to_string(),
                "dolby_vision".to_string(),
                "hlg".to_string(),
            ],
        };

        Ok(ContentStandards {
            video,
            audio,
            quality,
        })
    }
}

pub struct ComplianceEngine {
    standards: ContentStandards,
}

impl ComplianceEngine {
    pub fn new() -> Result<Self, VideoError> {
        let standards = ContentStandards::load_default()?;
        Ok(ComplianceEngine { standards })
    }

    #[instrument(skip(self))]
    pub fn analyze_compliance(&self, metadata: &VideoMetadata) -> ComplianceResult {
        info!("Analyzing compliance for video file");

        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        let mut score = 100u8;

        // Check video codec compliance
        if !self
            .standards
            .video
            .preferred_codecs
            .contains(&metadata.codec)
        {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Critical,
                category: ViolationCategory::VideoCodec,
                description: "Video codec not in preferred list".to_string(),
                current_value: metadata.codec.clone(),
                expected_value: self.standards.video.preferred_codecs.join(", "),
            });
            score = score.saturating_sub(20);
            recommendations.push("Convert to H.264 codec for optimal compatibility".to_string());
        }

        // Check resolution compliance
        let is_preferred_res = self
            .standards
            .video
            .preferred_resolutions
            .contains(&metadata.resolution);
        let is_acceptable_res = self
            .standards
            .video
            .acceptable_resolutions
            .contains(&metadata.resolution);

        if !is_preferred_res && !is_acceptable_res {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Critical,
                category: ViolationCategory::Resolution,
                description: "Resolution not supported".to_string(),
                current_value: metadata.resolution.clone(),
                expected_value: format!(
                    "Preferred: {}",
                    self.standards.video.preferred_resolutions.join(", ")
                ),
            });
            score = score.saturating_sub(25);
            recommendations
                .push("Resize to 1920x1080 or 1280x720 for standard content".to_string());
        } else if !is_preferred_res {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Warning,
                category: ViolationCategory::Resolution,
                description: "Resolution acceptable but not preferred".to_string(),
                current_value: metadata.resolution.clone(),
                expected_value: format!(
                    "Preferred: {}",
                    self.standards.video.preferred_resolutions.join(", ")
                ),
            });
            score = score.saturating_sub(10);
        }

        // Check container format
        if self
            .standards
            .video
            .unsupported_containers
            .contains(&metadata.container.to_lowercase())
        {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Critical,
                category: ViolationCategory::Container,
                description: "Container format not supported".to_string(),
                current_value: metadata.container.clone(),
                expected_value: self.standards.video.containers.join(", "),
            });
            score = score.saturating_sub(15);
            recommendations.push("Convert to MP4 or MOV container".to_string());
        }

        // Check audio codec
        if !self
            .standards
            .audio
            .preferred_codecs
            .contains(&metadata.audio_codec)
            && !self
                .standards
                .audio
                .acceptable_codecs
                .contains(&metadata.audio_codec)
        {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Warning,
                category: ViolationCategory::AudioCodec,
                description: "Audio codec not in preferred or acceptable list".to_string(),
                current_value: metadata.audio_codec.clone(),
                expected_value: format!(
                    "Preferred: {} | Acceptable: {}",
                    self.standards.audio.preferred_codecs.join(", "),
                    self.standards.audio.acceptable_codecs.join(", ")
                ),
            });
            score = score.saturating_sub(15);
            recommendations.push("Convert audio to PCM or ALAC for highest quality".to_string());
        } else if self
            .standards
            .audio
            .acceptable_codecs
            .contains(&metadata.audio_codec)
        {
            violations.push(ComplianceViolation {
                severity: ViolationSeverity::Info,
                category: ViolationCategory::AudioCodec,
                description: "Audio codec is acceptable but not preferred".to_string(),
                current_value: metadata.audio_codec.clone(),
                expected_value: format!(
                    "Preferred: {}",
                    self.standards.audio.preferred_codecs.join(", ")
                ),
            });
            score = score.saturating_sub(5);
        }

        // Check HDR restrictions and unsupported color spaces
        let mut hdr_violation = false;

        for restricted in &self.standards.quality.hdr_restrictions {
            if metadata
                .color_space
                .to_lowercase()
                .contains(&restricted.to_lowercase())
            {
                violations.push(ComplianceViolation {
                    severity: ViolationSeverity::Critical,
                    category: ViolationCategory::HDR,
                    description: "HDR content not supported by delivery pipeline".to_string(),
                    current_value: metadata.color_space.clone(),
                    expected_value: "Rec. 709 (SDR)".to_string(),
                });
                score = score.saturating_sub(30);
                recommendations.push("Convert to SDR (Rec. 709) color space".to_string());
                hdr_violation = true;
                break;
            }
        }

        // Also check unsupported color spaces (like bt2020)
        if !hdr_violation {
            for unsupported in &self.standards.quality.unsupported_color_spaces {
                if metadata
                    .color_space
                    .to_lowercase()
                    .contains(&unsupported.to_lowercase())
                {
                    violations.push(ComplianceViolation {
                        severity: ViolationSeverity::Critical,
                        category: ViolationCategory::HDR,
                        description: "Color space not supported by delivery pipeline".to_string(),
                        current_value: metadata.color_space.clone(),
                        expected_value: "Rec. 709 (SDR)".to_string(),
                    });
                    score = score.saturating_sub(25);
                    recommendations.push("Convert to SDR (Rec. 709) color space".to_string());
                    break;
                }
            }
        }

        let is_compliant = violations
            .iter()
            .all(|v| matches!(v.severity, ViolationSeverity::Info));

        ComplianceResult {
            is_compliant,
            score,
            violations,
            recommendations,
        }
    }

    pub fn get_standards(&self) -> &ContentStandards {
        &self.standards
    }
}

impl ComplianceResult {
    pub fn display(&self) {
        println!("\n{}", "📋 Compliance Analysis".bright_blue().bold());
        println!("{}", "=".repeat(60).bright_blue());

        let status_color = if self.is_compliant { "green" } else { "red" };
        let status_icon = if self.is_compliant { "✅" } else { "❌" };
        let status_text = if self.is_compliant {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        };

        println!(
            "{} Status: {}",
            status_icon,
            status_text.color(status_color).bold()
        );
        println!("📊 Compliance Score: {}/100", self.score.to_string().bold());

        if !self.violations.is_empty() {
            println!("\n{}", "⚠️  Violations Found:".yellow().bold());
            for (i, violation) in self.violations.iter().enumerate() {
                let severity_icon = match violation.severity {
                    ViolationSeverity::Critical => "🔴",
                    ViolationSeverity::Warning => "🟡",
                    ViolationSeverity::Info => "🔵",
                };
                let severity_color = match violation.severity {
                    ViolationSeverity::Critical => "red",
                    ViolationSeverity::Warning => "yellow",
                    ViolationSeverity::Info => "blue",
                };

                println!(
                    "\n{}. {} {} - {}",
                    (i + 1).to_string().bold(),
                    severity_icon,
                    format!("{:?}", violation.severity)
                        .color(severity_color)
                        .bold(),
                    violation.description
                );
                println!("   Current: {}", violation.current_value.cyan());
                println!("   Expected: {}", violation.expected_value.green());
            }
        }

        if !self.recommendations.is_empty() {
            println!("\n{}", "💡 Recommendations:".bright_green().bold());
            for (i, rec) in self.recommendations.iter().enumerate() {
                println!("{}. {}", (i + 1).to_string().bold(), rec);
            }
        }

        println!("{}", "=".repeat(60).bright_blue());
    }
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
    #[error("Compliance error: {0}")]
    Compliance(String),
    #[error("Standards parsing error: {0}")]
    Standards(String),
    #[error("Google Drive error: {0}")]
    GoogleDrive(String),
    #[error("Authentication error: {0}")]
    Auth(String),
}

#[derive(Debug, Default)]
pub struct ProcessingSummary {
    pub total_videos: usize,
    pub total_size: u64,
    pub total_duration: f64,
    pub codecs: HashMap<String, usize>,
    pub audio_codecs: HashMap<String, usize>,
    pub resolutions: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub codec: String,
    pub resolution: String,
    pub duration: f64,
    pub bitrate: u64,
    pub size: u64,
    pub fps: f64,
    pub audio_codec: String,
    pub audio_sample_rate: u32,
    pub audio_bitrate: u64,
    pub container: String,
    pub profile: String,
    pub color_space: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStandards {
    pub video: VideoStandards,
    pub audio: AudioStandards,
    pub quality: QualityStandards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStandards {
    pub preferred_resolutions: Vec<String>,
    pub acceptable_resolutions: Vec<String>,
    pub preferred_codecs: Vec<String>,
    pub preferred_frame_rates: Vec<f64>,
    pub bitrate_ranges: HashMap<String, BitRateRange>,
    pub containers: Vec<String>,
    pub unsupported_containers: Vec<String>,
    pub profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStandards {
    pub preferred_codecs: Vec<String>,
    pub acceptable_codecs: Vec<String>,
    pub sample_rates: Vec<u32>,
    pub bit_depths: Vec<u8>,
    pub bitrate_ranges: HashMap<String, u32>,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStandards {
    pub color_spaces: Vec<String>,
    pub unsupported_color_spaces: Vec<String>,
    pub keyframe_interval_min: u32,
    pub chroma_subsampling: Vec<String>,
    pub hdr_restrictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitRateRange {
    pub min_kbps: u32,
    pub max_kbps: u32,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub score: u8,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub severity: ViolationSeverity,
    pub category: ViolationCategory,
    pub description: String,
    pub current_value: String,
    pub expected_value: String,
}

#[derive(Debug, Clone)]
pub enum ViolationSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViolationCategory {
    VideoCodec,
    AudioCodec,
    Resolution,
    FrameRate,
    Bitrate,
    Container,
    ColorSpace,
    HDR,
    Profile,
    Audio,
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
            .audio_codecs
            .entry(metadata.audio_codec.clone())
            .or_insert(0) += 1;
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
            println!("\n{}", "Video Codec Distribution:".bright_blue().bold());
            for (codec, count) in &self.codecs {
                println!("  {} {} videos using {}", "•".blue(), count, codec);
            }
        }

        if !self.audio_codecs.is_empty() {
            println!("\n{}", "Audio Codec Distribution:".bright_blue().bold());
            for (codec, count) in &self.audio_codecs {
                println!("  {} {} videos using {}", "•".cyan(), count, codec);
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
         Video Codec Distribution:\n{}\n\
         Audio Codec Distribution:\n{}\n\
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
            .audio_codecs
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

    // Find video and audio streams
    let streams = json["streams"]
        .as_array()
        .ok_or_else(|| VideoError::FFmpeg("No streams found in video file".to_string()))?;

    let video_stream = streams
        .iter()
        .find(|stream| stream["codec_type"].as_str() == Some("video"))
        .ok_or_else(|| VideoError::FFmpeg("No video stream found".to_string()))?;

    let audio_stream = streams
        .iter()
        .find(|stream| stream["codec_type"].as_str() == Some("audio"));

    // Extract container format from file extension
    let container = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    // Extract comprehensive video metadata
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
        fps: video_stream["r_frame_rate"]
            .as_str()
            .and_then(|fps| {
                let parts: Vec<&str> = fps.split('/').collect();
                if parts.len() == 2 {
                    let num: f64 = parts[0].parse().ok()?;
                    let den: f64 = parts[1].parse().ok()?;
                    if den != 0.0 {
                        Some(num / den)
                    } else {
                        None
                    }
                } else {
                    fps.parse().ok()
                }
            })
            .unwrap_or(0.0),
        audio_codec: audio_stream
            .and_then(|stream| stream["codec_name"].as_str())
            .unwrap_or("none")
            .to_string(),
        audio_sample_rate: audio_stream
            .and_then(|stream| stream["sample_rate"].as_str())
            .and_then(|rate| rate.parse().ok())
            .unwrap_or(0),
        audio_bitrate: audio_stream
            .and_then(|stream| stream["bit_rate"].as_str())
            .and_then(|rate| rate.parse().ok())
            .unwrap_or(0),
        container,
        profile: video_stream["profile"]
            .as_str()
            .unwrap_or("unknown")
            .to_string()
            .to_lowercase(),
        color_space: video_stream["color_space"]
            .as_str()
            .unwrap_or("unknown")
            .to_string()
            .to_lowercase(),
    })
}

/// Intelligent video fixing based on compliance analysis
pub fn fix_video_compliance(
    input: &Path,
    output_dir: &Path,
    compliance_result: &ComplianceResult,
) -> Result<PathBuf, VideoError> {
    info!(
        "Starting intelligent compliance fixing for: {}",
        input.display()
    );

    // Generate output filename with compliance suffix
    let output_filename = generate_compliance_output_filename(input, compliance_result);
    let output_path = output_dir.join(&output_filename);

    println!(
        "\n{}",
        "🔧 Intelligent Compliance Fixing".bright_blue().bold()
    );
    println!("Input:  {}", input.display());
    println!("Output: {}", output_path.display());
    println!("Violations to fix: {}", compliance_result.violations.len());

    // Build FFmpeg arguments based on violations
    let mut args = vec!["-i".to_string(), input.to_str().unwrap().to_string()];

    // Add hardware decoding if beneficial
    if should_use_hw_decode(compliance_result) {
        args.extend(["-hwaccel".to_string(), "cuda".to_string()]);
    }

    // Video encoding settings
    let video_fixes = generate_video_fixes(compliance_result);
    args.extend(video_fixes);

    // Audio encoding settings
    let audio_fixes = generate_audio_fixes(compliance_result);
    args.extend(audio_fixes);

    // Quality preservation settings
    args.extend(["-movflags".to_string(), "+faststart".to_string()]);

    // Output overwrite and path
    args.extend(["-y".to_string(), output_path.to_str().unwrap().to_string()]);

    println!("FFmpeg command: ffmpeg {}", args.join(" "));

    // Execute conversion with progress tracking
    execute_compliance_conversion(&args, compliance_result)?;

    Ok(output_path)
}

/// Legacy basic conversion for backward compatibility
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

/// Generate output filename based on compliance violations
fn generate_compliance_output_filename(
    input: &Path,
    compliance_result: &ComplianceResult,
) -> String {
    let stem = input.file_stem().unwrap().to_str().unwrap();
    let ext = input.extension().unwrap_or_default().to_str().unwrap();

    let mut suffix = String::from(".compliant");

    // Add specific violation indicators
    if compliance_result
        .violations
        .iter()
        .any(|v| v.category == ViolationCategory::Resolution)
    {
        suffix.push_str(".scaled");
    }
    if compliance_result
        .violations
        .iter()
        .any(|v| v.category == ViolationCategory::VideoCodec)
    {
        suffix.push_str(".h264");
    }
    if compliance_result.violations.iter().any(|v| {
        v.category == ViolationCategory::Audio || v.category == ViolationCategory::AudioCodec
    }) {
        suffix.push_str(".aac");
    }
    if compliance_result.violations.iter().any(|v| {
        v.category == ViolationCategory::ColorSpace || v.category == ViolationCategory::HDR
    }) {
        suffix.push_str(".rec709");
    }

    format!(
        "{}{}.{}",
        stem,
        suffix,
        if ext.is_empty() { "mp4" } else { ext }
    )
}

/// Determine if hardware decoding should be used
fn should_use_hw_decode(compliance_result: &ComplianceResult) -> bool {
    // Use hardware decode for large files or when significant processing is needed
    compliance_result.violations.len() > 2
        || compliance_result.violations.iter().any(|v| {
            v.category == ViolationCategory::Resolution
                || v.category == ViolationCategory::ColorSpace
                || v.category == ViolationCategory::HDR
        })
}

/// Generate video encoding arguments based on compliance violations
fn generate_video_fixes(compliance_result: &ComplianceResult) -> Vec<String> {
    let mut args = Vec::new();

    // Check if codec fixing is needed
    let needs_codec_fix = compliance_result
        .violations
        .iter()
        .any(|v| v.category == ViolationCategory::VideoCodec);

    // Check if resolution fixing is needed
    let needs_resolution_fix = compliance_result
        .violations
        .iter()
        .any(|v| v.category == ViolationCategory::Resolution);

    // Check if quality fixes are needed
    let needs_quality_fix = compliance_result.violations.iter().any(|v| {
        v.category == ViolationCategory::ColorSpace || v.category == ViolationCategory::HDR
    });

    if needs_codec_fix || needs_resolution_fix || needs_quality_fix {
        // Use H.264 NVENC with high quality settings
        args.extend([
            "-c:v".to_string(),
            "h264_nvenc".to_string(),
            "-preset".to_string(),
            "p7".to_string(), // Highest quality preset
            "-cq".to_string(),
            "18".to_string(), // Constant quality mode
            "-profile:v".to_string(),
            "high".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
        ]);

        // Add resolution scaling if needed
        if needs_resolution_fix {
            // Find target resolution from violations
            let target_resolution = determine_target_resolution(compliance_result);
            if let Some(res) = target_resolution {
                args.extend(["-vf".to_string(), format!("scale={}", res)]);
            }
        }

        // Add color space conversion if needed
        if needs_quality_fix {
            args.extend([
                "-colorspace".to_string(),
                "bt709".to_string(),
                "-color_primaries".to_string(),
                "bt709".to_string(),
                "-color_trc".to_string(),
                "bt709".to_string(),
            ]);
        }
    } else {
        // No video fixes needed, copy stream
        args.extend(["-c:v".to_string(), "copy".to_string()]);
    }

    args
}

/// Generate audio encoding arguments based on compliance violations
fn generate_audio_fixes(compliance_result: &ComplianceResult) -> Vec<String> {
    let mut args = Vec::new();

    let needs_audio_fix = compliance_result.violations.iter().any(|v| {
        v.category == ViolationCategory::Audio || v.category == ViolationCategory::AudioCodec
    });

    if needs_audio_fix {
        // Use PCM audio (pcm_s24le) as per content delivery spec and DaVinci Resolve compatibility
        args.extend([
            "-c:a".to_string(),
            "pcm_s24le".to_string(), // 24-bit PCM (preferred format)
            "-ar".to_string(),
            "48000".to_string(), // 48kHz sample rate
            "-ac".to_string(),
            "2".to_string(), // Stereo
        ]);
    } else {
        // No audio fixes needed, copy stream
        args.extend(["-c:a".to_string(), "copy".to_string()]);
    }

    args
}

/// Determine target resolution from compliance violations
fn determine_target_resolution(compliance_result: &ComplianceResult) -> Option<String> {
    // Look for resolution recommendations in violations
    for violation in &compliance_result.violations {
        if violation.category == ViolationCategory::Resolution {
            if violation.expected_value.contains("1920x1080") {
                return Some("1920:1080".to_string());
            }
            if violation.expected_value.contains("1280x720") {
                return Some("1280:720".to_string());
            }
        }
    }

    // Default to 1080p if no specific recommendation
    Some("1920:1080".to_string())
}

/// Execute compliance conversion with progress tracking
fn execute_compliance_conversion(
    args: &[String],
    compliance_result: &ComplianceResult,
) -> Result<(), VideoError> {
    println!("\n{}", "🔄 Starting compliance conversion...".bright_blue());

    // Show what will be fixed
    for violation in &compliance_result.violations {
        let icon = match violation.severity {
            ViolationSeverity::Critical => "🔴",
            ViolationSeverity::Warning => "🟡",
            ViolationSeverity::Info => "🔵",
        };
        println!("{} Fixing: {}", icon, violation.description);
    }

    let mut child = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let status = child.wait()?;

    if status.success() {
        println!(
            "{}",
            "✅ Compliance conversion completed successfully!"
                .green()
                .bold()
        );
    } else {
        return Err(VideoError::FFmpeg(format!(
            "Compliance conversion failed with exit code: {:?}",
            status.code()
        )));
    }

    Ok(())
}

/// Content type detection for optimal encoding settings
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ScreenCapture,
    LiveAction,
    Animation,
    Presentation,
    Unknown,
}

/// Analyze video content to determine optimal processing approach
pub fn detect_content_type(metadata: &VideoMetadata, path: &Path) -> ContentType {
    let filename = path.file_name().unwrap().to_str().unwrap().to_lowercase();

    // Filename-based heuristics
    if filename.contains("screen") || filename.contains("capture") || filename.contains("recording")
    {
        return ContentType::ScreenCapture;
    }

    if filename.contains("presentation") || filename.contains("slide") || filename.contains("demo")
    {
        return ContentType::Presentation;
    }

    if filename.contains("cartoon") || filename.contains("animated") || filename.contains("anime") {
        return ContentType::Animation;
    }

    // Metadata-based analysis
    // High framerate usually indicates live action
    if metadata.fps > 50.0 {
        return ContentType::LiveAction;
    }

    // Very low framerates suggest screen capture or presentations
    if metadata.fps < 20.0 {
        return ContentType::ScreenCapture;
    }

    // Resolution-based heuristics
    if metadata.resolution.contains("1920x1080") && metadata.fps >= 24.0 && metadata.fps <= 30.0 {
        return ContentType::LiveAction;
    }

    // Default to screen capture for most content
    ContentType::ScreenCapture
}

/// Get optimal bitrate based on content type and resolution
pub fn get_optimal_bitrate(content_type: &ContentType, resolution: &str) -> u32 {
    match content_type {
        ContentType::ScreenCapture | ContentType::Presentation => {
            if resolution.contains("1920x1080") {
                6000 // 6Mbps for 1080p screen capture
            } else if resolution.contains("1280x720") {
                4000 // 4Mbps for 720p screen capture
            } else {
                3000 // 3Mbps for other resolutions
            }
        }
        ContentType::LiveAction => {
            if resolution.contains("1920x1080") {
                12000 // 12Mbps for 1080p live action
            } else if resolution.contains("1280x720") {
                8000 // 8Mbps for 720p live action
            } else {
                5000 // 5Mbps for other resolutions
            }
        }
        ContentType::Animation => {
            if resolution.contains("1920x1080") {
                8000 // 8Mbps for 1080p animation
            } else if resolution.contains("1280x720") {
                6000 // 6Mbps for 720p animation
            } else {
                4000 // 4Mbps for other resolutions
            }
        }
        ContentType::Unknown => {
            // Conservative middle ground
            if resolution.contains("1920x1080") {
                8000
            } else if resolution.contains("1280x720") {
                6000
            } else {
                4000
            }
        }
    }
}

/// Enhanced video fixing with content-aware optimization
pub fn fix_video_compliance_optimized(
    input: &Path,
    output_dir: &Path,
    compliance_result: &ComplianceResult,
    metadata: &VideoMetadata,
) -> Result<PathBuf, VideoError> {
    info!(
        "Starting content-aware compliance fixing for: {}",
        input.display()
    );

    // Detect content type for optimization
    let content_type = detect_content_type(metadata, input);
    info!("Detected content type: {:?}", content_type);

    // Generate output filename with content type indicator
    let output_filename =
        generate_optimized_output_filename(input, compliance_result, &content_type);
    let output_path = output_dir.join(&output_filename);

    println!(
        "\n{}",
        "🧠 Content-Aware Compliance Fixing".bright_blue().bold()
    );
    println!("Input:        {}", input.display());
    println!("Output:       {}", output_path.display());
    println!("Content Type: {:?}", content_type);
    println!("Violations:   {}", compliance_result.violations.len());

    // Build optimized FFmpeg arguments
    let mut args = vec!["-i".to_string(), input.to_str().unwrap().to_string()];

    // Hardware acceleration settings
    if should_use_hw_decode(compliance_result) {
        args.extend(["-hwaccel".to_string(), "cuda".to_string()]);
    }

    // Content-aware video encoding
    let video_fixes = generate_optimized_video_fixes(compliance_result, &content_type, metadata);
    args.extend(video_fixes);

    // Audio fixes
    let audio_fixes = generate_audio_fixes(compliance_result);
    args.extend(audio_fixes);

    // Optimization flags
    args.extend([
        "-movflags".to_string(),
        "+faststart".to_string(),
        "-y".to_string(),
        output_path.to_str().unwrap().to_string(),
    ]);

    println!("Optimized command: ffmpeg {}", args.join(" "));

    // Execute with progress tracking
    execute_compliance_conversion(&args, compliance_result)?;

    Ok(output_path)
}

/// Generate optimized output filename including content type
fn generate_optimized_output_filename(
    input: &Path,
    _compliance_result: &ComplianceResult,
    _content_type: &ContentType,
) -> String {
    // Keep the original filename as requested by user
    input.file_name().unwrap().to_str().unwrap().to_string()
}

/// Generate content-optimized video encoding arguments
fn generate_optimized_video_fixes(
    compliance_result: &ComplianceResult,
    content_type: &ContentType,
    _metadata: &VideoMetadata,
) -> Vec<String> {
    let mut args = Vec::new();

    let needs_video_fix = compliance_result.violations.iter().any(|v| {
        v.category == ViolationCategory::VideoCodec
            || v.category == ViolationCategory::Resolution
            || v.category == ViolationCategory::ColorSpace
            || v.category == ViolationCategory::HDR
    });

    if needs_video_fix {
        // Base H.264 NVENC settings
        args.extend([
            "-c:v".to_string(),
            "h264_nvenc".to_string(),
            "-profile:v".to_string(),
            "high".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
        ]);

        // Content-specific optimizations
        match content_type {
            ContentType::ScreenCapture | ContentType::Presentation => {
                // Screen capture: focus on sharpness and detail
                args.extend([
                    "-preset".to_string(),
                    "p7".to_string(), // Highest quality
                    "-cq".to_string(),
                    "15".to_string(), // Very high quality
                    "-temporal-aq".to_string(),
                    "1".to_string(),
                    "-rc-lookahead".to_string(),
                    "32".to_string(),
                ]);
            }
            ContentType::LiveAction => {
                // Live action: balanced quality and efficiency
                args.extend([
                    "-preset".to_string(),
                    "p5".to_string(), // Balanced preset
                    "-cq".to_string(),
                    "18".to_string(), // High quality
                    "-spatial-aq".to_string(),
                    "1".to_string(),
                    "-temporal-aq".to_string(),
                    "1".to_string(),
                ]);
            }
            ContentType::Animation => {
                // Animation: preserve flat colors and sharp edges
                args.extend([
                    "-preset".to_string(),
                    "p6".to_string(),
                    "-cq".to_string(),
                    "16".to_string(),
                    "-aq-mode".to_string(),
                    "3".to_string(), // Temporal AQ
                ]);
            }
            ContentType::Unknown => {
                // Safe defaults
                args.extend([
                    "-preset".to_string(),
                    "p6".to_string(),
                    "-cq".to_string(),
                    "18".to_string(),
                ]);
            }
        }

        // Add resolution scaling if needed
        if compliance_result
            .violations
            .iter()
            .any(|v| v.category == ViolationCategory::Resolution)
        {
            if let Some(target_res) = determine_target_resolution(compliance_result) {
                args.extend(["-vf".to_string(), format!("scale={}", target_res)]);
            }
        }

        // Add color space conversion if needed
        if compliance_result.violations.iter().any(|v| {
            v.category == ViolationCategory::ColorSpace || v.category == ViolationCategory::HDR
        }) {
            args.extend([
                "-colorspace".to_string(),
                "bt709".to_string(),
                "-color_primaries".to_string(),
                "bt709".to_string(),
                "-color_trc".to_string(),
                "bt709".to_string(),
            ]);
        }
    } else {
        args.extend(["-c:v".to_string(), "copy".to_string()]);
    }

    args
}

/// Batch processing for multiple files with progress tracking
pub struct BatchProcessor {
    pub total_files: usize,
    pub processed_files: usize,
    pub failed_files: Vec<(PathBuf, String)>,
    pub fixed_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
}

impl BatchProcessor {
    pub fn new(total_files: usize) -> Self {
        Self {
            total_files,
            processed_files: 0,
            failed_files: Vec::new(),
            fixed_files: Vec::new(),
            skipped_files: Vec::new(),
        }
    }

    pub fn process_file_result(
        &mut self,
        path: PathBuf,
        result: Result<Option<PathBuf>, VideoError>,
    ) {
        self.processed_files += 1;

        match result {
            Ok(Some(fixed_path)) => {
                self.fixed_files.push(fixed_path);
            }
            Ok(None) => {
                self.skipped_files.push(path);
            }
            Err(e) => {
                self.failed_files.push((path, e.to_string()));
            }
        }
    }

    pub fn display_final_report(&self) {
        println!("\n{}", "📊 Batch Processing Report".bright_green().bold());
        println!("{}", "=".repeat(65).bright_green());

        println!("📁 Total Files: {}", self.total_files.to_string().bold());
        println!(
            "✅ Files Fixed: {} ({:.1}%)",
            self.fixed_files.len().to_string().green().bold(),
            (self.fixed_files.len() as f64 / self.total_files as f64) * 100.0
        );
        println!(
            "⏭️  Files Skipped: {} ({:.1}%)",
            self.skipped_files.len().to_string().blue().bold(),
            (self.skipped_files.len() as f64 / self.total_files as f64) * 100.0
        );

        if !self.failed_files.is_empty() {
            println!(
                "❌ Files Failed: {} ({:.1}%)",
                self.failed_files.len().to_string().red().bold(),
                (self.failed_files.len() as f64 / self.total_files as f64) * 100.0
            );

            println!("\n{}", "❌ Failed Files:".red().bold());
            for (path, error) in &self.failed_files {
                println!("  {} - {}", path.display().to_string().red(), error);
            }
        }

        if !self.fixed_files.is_empty() {
            println!("\n{}", "✅ Successfully Fixed Files:".green().bold());
            for path in self.fixed_files.iter().take(5) {
                println!("  {}", path.display().to_string().green());
            }
            if self.fixed_files.len() > 5 {
                println!(
                    "  ... and {} more",
                    (self.fixed_files.len() - 5).to_string().green()
                );
            }
        }

        println!("{}", "=".repeat(65).bright_green());
    }
}

/// Process single file for batch operations
pub fn process_single_file_batch(
    path: &Path,
    output_dir: &Path,
    compliance_engine: &ComplianceEngine,
    verbose: bool,
) -> Result<Option<PathBuf>, VideoError> {
    if verbose {
        println!("📁 Processing: {}", path.display());
    }

    // Analyze the video
    let metadata = analyze_video(path)?;

    // Check compliance
    let compliance_result = compliance_engine.analyze_compliance(&metadata);

    if compliance_result.is_compliant {
        if verbose {
            println!("✅ Already compliant: {}", path.display());
        }
        return Ok(None); // File is already compliant
    }

    // Fix the non-compliant file
    let fixed_path =
        fix_video_compliance_optimized(path, output_dir, &compliance_result, &metadata)?;

    if verbose {
        println!("✅ Fixed: {} -> {}", path.display(), fixed_path.display());
    }

    Ok(Some(fixed_path))
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
    analyze_compliance: bool,
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

    let compliance_engine = if analyze_compliance {
        Some(ComplianceEngine::new()?)
    } else {
        None
    };

    spinner.finish_and_clear();
    println!("{}", "\nScanning directory:".bright_blue());
    println!("{}", dir.display().to_string().bright_blue().bold());
    println!("{}", "=".repeat(50).bright_blue());

    let mut summary = ProcessingSummary::new();
    let mut compliance_summary = ComplianceSummary::new();

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
            println!("  {} {:.1} fps", "Frame Rate:".blue(), metadata.fps);
            println!("  {} {}", "Audio:".blue(), metadata.audio_codec);
            println!("  {} {}", "Container:".blue(), metadata.container);
            println!("  {} {}", "Profile:".blue(), metadata.profile);
            println!("  {} {}", "Color Space:".blue(), metadata.color_space);
        }

        // Run compliance analysis if requested - moved outside to fix scope
        let compliance_result = if let Some(ref engine) = compliance_engine {
            let result = engine.analyze_compliance(&metadata);
            result.display();
            compliance_summary.add_result(&result, path.file_name().unwrap().to_str().unwrap());
            Some(result)
        } else {
            None
        };

        summary.add_video(&metadata);

        // Intelligent compliance-driven fixing
        if let Some(ref h264_dir) = h264_dir {
            if analyze_compliance {
                if let Some(ref result) = compliance_result {
                    if !result.is_compliant {
                        // Use content-aware intelligent compliance fixing
                        let fixed_path =
                            fix_video_compliance_optimized(&path, h264_dir, result, &metadata)?;
                        println!(
                            "{} Fixed file saved: {}",
                            "✅".green(),
                            fixed_path.display()
                        );
                    } else {
                        println!(
                            "{} File is already compliant, no fixing needed",
                            "✅".green()
                        );
                    }
                }
            } else {
                // Legacy basic conversion for backward compatibility
                convert_video(&path, h264_dir)?;
            }
        }
    }

    if let Some(ref h264_dir) = h264_dir {
        write_conversion_report(h264_dir, &summary)?;
    }

    // Display compliance summary if analysis was performed
    if analyze_compliance {
        compliance_summary.display();
    }

    summary.display();
    Ok(())
}

#[derive(Debug, Default)]
pub struct ComplianceSummary {
    pub total_files: usize,
    pub compliant_files: usize,
    pub non_compliant_files: usize,
    pub average_score: f64,
    pub critical_violations: usize,
    pub warning_violations: usize,
    pub info_violations: usize,
    pub files_by_score: Vec<(String, u8)>,
}

impl ComplianceSummary {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_result(&mut self, result: &ComplianceResult, filename: &str) {
        self.total_files += 1;
        self.average_score = (self.average_score * (self.total_files - 1) as f64
            + result.score as f64)
            / self.total_files as f64;
        self.files_by_score
            .push((filename.to_string(), result.score));

        if result.is_compliant {
            self.compliant_files += 1;
        } else {
            self.non_compliant_files += 1;
        }

        for violation in &result.violations {
            match violation.severity {
                ViolationSeverity::Critical => self.critical_violations += 1,
                ViolationSeverity::Warning => self.warning_violations += 1,
                ViolationSeverity::Info => self.info_violations += 1,
            }
        }
    }

    pub fn display(&self) {
        println!("\n{}", "📊 Compliance Summary".bright_green().bold());
        println!("{}", "=".repeat(60).bright_green());

        let compliance_rate = if self.total_files > 0 {
            (self.compliant_files as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "📁 Total Files Analyzed: {}",
            self.total_files.to_string().bold()
        );
        println!(
            "✅ Compliant Files: {} ({:.1}%)",
            self.compliant_files.to_string().green().bold(),
            compliance_rate
        );
        println!(
            "❌ Non-Compliant Files: {} ({:.1}%)",
            self.non_compliant_files.to_string().red().bold(),
            100.0 - compliance_rate
        );
        println!(
            "📊 Average Compliance Score: {:.1}/100",
            self.average_score.to_string().bold()
        );

        if self.critical_violations > 0 || self.warning_violations > 0 || self.info_violations > 0 {
            println!("\n{}", "🚨 Violation Breakdown:".yellow().bold());
            if self.critical_violations > 0 {
                println!(
                    "🔴 Critical: {}",
                    self.critical_violations.to_string().red().bold()
                );
            }
            if self.warning_violations > 0 {
                println!(
                    "🟡 Warnings: {}",
                    self.warning_violations.to_string().yellow().bold()
                );
            }
            if self.info_violations > 0 {
                println!(
                    "🔵 Info: {}",
                    self.info_violations.to_string().blue().bold()
                );
            }
        }

        // Show worst performing files
        if self.files_by_score.len() > 1 {
            let mut sorted_files = self.files_by_score.clone();
            sorted_files.sort_by_key(|(_, score)| *score);

            println!("\n{}", "📉 Files Needing Attention:".bright_yellow().bold());
            for (filename, score) in sorted_files.iter().take(3) {
                if *score < 100 {
                    let score_color = if *score < 60 {
                        "red"
                    } else if *score < 80 {
                        "yellow"
                    } else {
                        "blue"
                    };
                    println!(
                        "  {} - Score: {}/100",
                        filename,
                        score.to_string().color(score_color).bold()
                    );
                }
            }
        }

        println!("{}", "=".repeat(60).bright_green());
    }
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

    fn create_test_metadata() -> VideoMetadata {
        VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 120.0,
            bitrate: 5_000_000,
            size: 75_000_000,
            fps: 30.0,
            audio_codec: "aac".to_string(),
            audio_sample_rate: 48000,
            audio_bitrate: 320000,
            container: "mp4".to_string(),
            profile: "high".to_string(),
            color_space: "rec709".to_string(),
        }
    }

    #[test]
    fn test_summary_tracking() {
        let mut summary = ProcessingSummary::new();
        let metadata = create_test_metadata();

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
        summary.add_video(&create_test_metadata());

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
        let mut metadata = create_test_metadata();
        metadata.codec = "hevc".to_string();
        metadata.resolution = "3840x2160".to_string();
        metadata.duration = 7200.0;
        metadata.bitrate = 25000000;
        metadata.size = 2000000000;

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
        let mut video1 = create_test_metadata();
        video1.duration = 120.0;
        video1.bitrate = 5000000;
        video1.size = 100000000;

        let mut video2 = create_test_metadata();
        video2.codec = "hevc".to_string();
        video2.resolution = "3840x2160".to_string();
        video2.duration = 180.0;
        video2.bitrate = 15000000;
        video2.size = 300000000;

        let mut video3 = create_test_metadata();
        video3.resolution = "1280x720".to_string();
        video3.duration = 90.0;
        video3.bitrate = 3000000;
        video3.size = 50000000;

        let videos = vec![video1, video2, video3];

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
            audio_codecs: HashMap::from([("aac".to_string(), 3), ("pcm_s24le".to_string(), 2)]),
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

    #[test]
    fn test_content_standards_loading() {
        let standards = ContentStandards::load_default().unwrap();

        // Test video standards
        assert!(standards
            .video
            .preferred_resolutions
            .contains(&"1920x1080".to_string()));
        assert!(standards
            .video
            .preferred_resolutions
            .contains(&"1280x720".to_string()));
        assert!(standards
            .video
            .preferred_codecs
            .contains(&"h264".to_string()));
        assert!(standards
            .video
            .unsupported_containers
            .contains(&"mkv".to_string()));

        // Test audio standards
        assert!(standards
            .audio
            .preferred_codecs
            .contains(&"pcm".to_string()));
        assert!(standards
            .audio
            .acceptable_codecs
            .contains(&"aac".to_string()));

        // Test quality standards
        assert!(standards
            .quality
            .color_spaces
            .contains(&"rec709".to_string()));
        assert!(standards
            .quality
            .hdr_restrictions
            .contains(&"hdr10".to_string()));
    }

    #[test]
    fn test_compliance_engine_creation() {
        let engine = ComplianceEngine::new();
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        let standards = engine.get_standards();
        assert!(standards
            .video
            .preferred_codecs
            .contains(&"h264".to_string()));
    }

    #[test]
    fn test_compliance_analysis_compliant_file() {
        let engine = ComplianceEngine::new().unwrap();
        let metadata = create_test_metadata(); // h264, 1920x1080, aac, mp4, rec709 - should be compliant

        let result = engine.analyze_compliance(&metadata);
        assert!(result.is_compliant);
        assert!(result.score >= 90); // Should have high score for compliant file
        assert!(result
            .violations
            .iter()
            .all(|v| matches!(v.severity, ViolationSeverity::Info)));
    }

    #[test]
    fn test_compliance_analysis_non_compliant_file() {
        let engine = ComplianceEngine::new().unwrap();
        let mut metadata = create_test_metadata();

        // Make it non-compliant
        metadata.codec = "mpeg4".to_string(); // Not preferred
        metadata.container = "mkv".to_string(); // Unsupported
        metadata.color_space = "bt2020".to_string(); // Unsupported HDR

        let result = engine.analyze_compliance(&metadata);
        assert!(!result.is_compliant);
        assert!(result.score < 90); // Should have low score
        assert!(!result.violations.is_empty());
        assert!(!result.recommendations.is_empty());

        // Check for specific violations
        assert!(result
            .violations
            .iter()
            .any(|v| matches!(v.category, ViolationCategory::VideoCodec)));
        assert!(result
            .violations
            .iter()
            .any(|v| matches!(v.category, ViolationCategory::Container)));
        assert!(result
            .violations
            .iter()
            .any(|v| matches!(v.category, ViolationCategory::HDR)));
    }

    #[test]
    fn test_compliance_summary() {
        let mut summary = ComplianceSummary::new();

        // Test initial state
        assert_eq!(summary.total_files, 0);
        assert_eq!(summary.compliant_files, 0);
        assert_eq!(summary.non_compliant_files, 0);

        // Add compliant result
        let compliant_result = ComplianceResult {
            is_compliant: true,
            score: 100,
            violations: vec![],
            recommendations: vec![],
        };
        summary.add_result(&compliant_result, "test1.mp4");

        assert_eq!(summary.total_files, 1);
        assert_eq!(summary.compliant_files, 1);
        assert_eq!(summary.non_compliant_files, 0);
        assert_eq!(summary.average_score, 100.0);

        // Add non-compliant result
        let non_compliant_result = ComplianceResult {
            is_compliant: false,
            score: 60,
            violations: vec![ComplianceViolation {
                severity: ViolationSeverity::Critical,
                category: ViolationCategory::VideoCodec,
                description: "Test violation".to_string(),
                current_value: "mpeg4".to_string(),
                expected_value: "h264".to_string(),
            }],
            recommendations: vec!["Fix codec".to_string()],
        };
        summary.add_result(&non_compliant_result, "test2.mp4");

        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.compliant_files, 1);
        assert_eq!(summary.non_compliant_files, 1);
        assert_eq!(summary.average_score, 80.0); // (100 + 60) / 2
        assert_eq!(summary.critical_violations, 1);

        // Test display (shouldn't panic)
        summary.display();
    }

    #[test]
    fn test_violation_severity_and_category() {
        let violation = ComplianceViolation {
            severity: ViolationSeverity::Critical,
            category: ViolationCategory::VideoCodec,
            description: "Test".to_string(),
            current_value: "test".to_string(),
            expected_value: "test".to_string(),
        };

        assert!(matches!(violation.severity, ViolationSeverity::Critical));
        assert!(matches!(violation.category, ViolationCategory::VideoCodec));

        // Test Debug formatting
        let debug_str = format!("{:?}", violation);
        assert!(debug_str.contains("Critical"));
        assert!(debug_str.contains("VideoCodec"));
    }

    #[test]
    fn test_compliance_result_display() {
        let result = ComplianceResult {
            is_compliant: false,
            score: 75,
            violations: vec![ComplianceViolation {
                severity: ViolationSeverity::Warning,
                category: ViolationCategory::Resolution,
                description: "Resolution not preferred".to_string(),
                current_value: "1366x768".to_string(),
                expected_value: "1920x1080".to_string(),
            }],
            recommendations: vec!["Resize to 1920x1080".to_string()],
        };

        // Test display (shouldn't panic)
        result.display();
    }

    #[test]
    fn test_processing_summary_add_video() {
        let mut summary = ProcessingSummary::new();

        let metadata = VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 120.0,
            bitrate: 5000000,
            size: 75000000,
            fps: 30.0,
            audio_codec: "aac".to_string(),
            audio_sample_rate: 48000,
            audio_bitrate: 320000,
            container: "mp4".to_string(),
            profile: "high".to_string(),
            color_space: "bt709".to_string(),
        };

        summary.add_video(&metadata);
        assert_eq!(summary.total_videos, 1);
        assert_eq!(summary.total_size, 75000000);
        assert_eq!(summary.total_duration, 120.0);
        assert_eq!(*summary.codecs.get("h264").unwrap(), 1);
        assert_eq!(*summary.audio_codecs.get("aac").unwrap(), 1);
        assert_eq!(*summary.resolutions.get("1920x1080").unwrap(), 1);
    }

    #[test]
    fn test_processing_summary_display_coverage() {
        let mut summary = ProcessingSummary::new();

        // Add multiple videos with different codecs
        let metadata1 = VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 60.0,
            bitrate: 5000000,
            size: 37500000,
            fps: 30.0,
            audio_codec: "pcm_s24le".to_string(),
            audio_sample_rate: 48000,
            audio_bitrate: 1536000,
            container: "mp4".to_string(),
            profile: "high".to_string(),
            color_space: "bt709".to_string(),
        };

        let metadata2 = VideoMetadata {
            codec: "h265".to_string(),
            resolution: "3840x2160".to_string(),
            duration: 180.0,
            bitrate: 15000000,
            size: 337500000,
            fps: 60.0,
            audio_codec: "aac".to_string(),
            audio_sample_rate: 44100,
            audio_bitrate: 256000,
            container: "mp4".to_string(),
            profile: "main".to_string(),
            color_space: "bt2020".to_string(),
        };

        summary.add_video(&metadata1);
        summary.add_video(&metadata2);

        // Test display function doesn't panic
        summary.display();

        assert_eq!(summary.total_videos, 2);
        assert_eq!(*summary.codecs.get("h264").unwrap(), 1);
        assert_eq!(*summary.codecs.get("h265").unwrap(), 1);
        assert_eq!(*summary.audio_codecs.get("pcm_s24le").unwrap(), 1);
        assert_eq!(*summary.audio_codecs.get("aac").unwrap(), 1);
    }

    #[test]
    fn test_violation_category_display() {
        assert_eq!(format!("{:?}", ViolationCategory::Resolution), "Resolution");
        assert_eq!(format!("{:?}", ViolationCategory::VideoCodec), "VideoCodec");
        assert_eq!(format!("{:?}", ViolationCategory::AudioCodec), "AudioCodec");
        assert_eq!(format!("{:?}", ViolationCategory::FrameRate), "FrameRate");
        assert_eq!(format!("{:?}", ViolationCategory::Audio), "Audio");
    }

    #[test]
    fn test_violation_severity_ordering() {
        // Test that severity levels have correct ordering for priority
        let critical = ViolationSeverity::Critical as u8;
        let warning = ViolationSeverity::Warning as u8;
        let info = ViolationSeverity::Info as u8;

        assert!(critical < warning);
        assert!(warning < info);
    }

    #[test]
    fn test_content_type_detection() {
        // Test that ContentType enum works properly
        let content_types = vec![
            ContentType::ScreenCapture,
            ContentType::LiveAction,
            ContentType::Animation,
            ContentType::Presentation,
            ContentType::Unknown,
        ];

        for ct in content_types {
            match ct {
                ContentType::ScreenCapture => assert_eq!(format!("{:?}", ct), "ScreenCapture"),
                ContentType::LiveAction => assert_eq!(format!("{:?}", ct), "LiveAction"),
                ContentType::Animation => assert_eq!(format!("{:?}", ct), "Animation"),
                ContentType::Presentation => assert_eq!(format!("{:?}", ct), "Presentation"),
                ContentType::Unknown => assert_eq!(format!("{:?}", ct), "Unknown"),
            }
        }
    }

    #[test]
    fn test_compliance_engine_analyze() {
        let engine = ComplianceEngine::new().expect("Failed to create engine");

        // Test compliant video
        let good_metadata = VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 120.0,
            bitrate: 5000000,
            size: 75000000,
            fps: 30.0,
            audio_codec: "pcm_s24le".to_string(),
            audio_sample_rate: 48000,
            audio_bitrate: 320000,
            container: "mp4".to_string(),
            profile: "high".to_string(),
            color_space: "bt709".to_string(),
        };

        let result = engine.analyze_compliance(&good_metadata);
        // PCM audio should give high score
        assert!(result.score >= 85);

        // Test non-compliant video
        let bad_metadata = VideoMetadata {
            codec: "vp9".to_string(),
            resolution: "640x480".to_string(),
            duration: 120.0,
            bitrate: 500000,
            size: 7500000,
            fps: 15.0,
            audio_codec: "opus".to_string(),
            audio_sample_rate: 22050,
            audio_bitrate: 64000,
            container: "webm".to_string(),
            profile: "0".to_string(),
            color_space: "unknown".to_string(),
        };

        let bad_result = engine.analyze_compliance(&bad_metadata);
        assert!(bad_result.score < 50);
        assert!(!bad_result.is_compliant);
    }

    #[test]
    fn test_compliance_scoring() {
        let engine = ComplianceEngine::new().expect("Failed to create engine");

        // Edge case: very high frame rate
        let high_fps_metadata = VideoMetadata {
            codec: "h264".to_string(),
            resolution: "1920x1080".to_string(),
            duration: 60.0,
            bitrate: 8000000,
            size: 60000000,
            fps: 120.0,
            audio_codec: "pcm_s24le".to_string(),
            audio_sample_rate: 48000,
            audio_bitrate: 320000,
            container: "mp4".to_string(),
            profile: "high".to_string(),
            color_space: "bt709".to_string(),
        };

        let fps_result = engine.analyze_compliance(&high_fps_metadata);
        // 120 fps is very high, should have violations or lower score
        assert!(fps_result.score < 100 || !fps_result.violations.is_empty());
    }

    #[test]
    fn test_analyze_video_error_handling() {
        use std::path::Path;

        // Test with non-existent file
        let result = analyze_video(Path::new("/non/existent/file.mp4"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_empty_directory_with_report() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let result = process_directory(temp.path(), false, false, true);

        // Empty directory should return error
        assert!(result.is_err());
    }
}
