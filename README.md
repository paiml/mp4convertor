# Video Content Standards Compliance System

[![TDG Score](https://img.shields.io/badge/TDG-92.7%2F100%20(A)-brightgreen)](https://github.com/paiml/paiml-mcp-agent-toolkit)
[![Coverage](https://img.shields.io/badge/Coverage-80.45%25-brightgreen)](https://github.com/taiki-e/cargo-llvm-cov)
[![Quality](https://img.shields.io/badge/Quality-A%20Grade-brightgreen)](https://github.com/paiml/paiml-mcp-agent-toolkit)
[![Cloud](https://img.shields.io/badge/Google_Drive-Integrated-blue)](https://developers.google.com/drive)

Intelligent video compliance system built in Rust for professional content delivery standards. Automatically analyzes, audits, and fixes video files across local storage and Google Drive to meet industry specifications.

## 🚀 Features

### 🔍 **Compliance Analysis**
- **📋 Standards Validation**: Enforces professional content delivery specifications
- **⚡ Real-time Scoring**: 0-100 compliance scores with detailed violation reports
- **🎯 Multi-format Support**: MP4, AVI, MOV analysis with comprehensive metadata extraction
- **📊 Enhanced Reporting**: Detailed summaries with video AND audio codec distribution tracking
- **🎬 Smart Recommendations**: Actionable fix suggestions based on content standards

### ☁️ **Google Drive Integration**
- **🔐 OAuth2 Authentication**: Secure Google Drive API integration (demo mode)
- **📁 Cloud Discovery**: Automatic detection and analysis of Drive-hosted videos
- **⬇️ Smart Downloads**: Efficient cloud file processing without permanent storage
- **📈 Remote Auditing**: Comprehensive compliance reports for cloud assets
- **🔄 Batch Processing**: Analyze multiple cloud files simultaneously

### 🛠️ **Intelligent Fixing**
- **🎯 GPU Acceleration**: Fast video conversion using NVIDIA NVENC hardware encoding
- **🔄 Format Conversion**: Automatic transcoding to compliant H.264 formats
- **📐 Resolution Optimization**: Smart scaling to preferred resolutions (1080p/720p)
- **🎵 Audio Enhancement**: PCM/ALAC encoding for maximum quality
- **🛡️ Non-destructive**: Original files are never modified or overwritten

## 🏆 Quality Standards

This project maintains **industry-leading quality standards**:

- **TDG Score**: 92.7/100 (A grade) - Technical Debt Grade via pmat analysis
- **Test Coverage**: 81.76% line coverage with 38 comprehensive tests
- **Zero Technical Debt**: No TODO, FIXME, or HACK markers allowed
- **Security**: Zero known vulnerabilities via cargo audit
- **Pre-commit Hooks**: Automatic quality gate enforcement on every commit

## 📋 Requirements

### Hardware
- **NVIDIA GPU** with NVENC support (GTX 10-series or newer)
- **CUDA drivers** installed and working

### Software
- **Rust toolchain** (2021 edition)
- **FFmpeg** with NVENC support compiled
- **NVIDIA CUDA toolkit** for hardware acceleration

### Optional Development Tools
- **cargo-llvm-cov** for coverage reports
- **pmat** for TDG analysis and quality gates

## 🔧 Installation

### From Crates.io (Recommended)
```bash
# Install the latest version from crates.io
cargo install mp4converter

# Verify installation
mp4converter --version
```

### From Source
```bash
# Clone and install from source
git clone https://github.com/paiml/mp4convertor.git
cd mp4converter
make install
```

### Development Setup
```bash
# Install with development tools and quality gates
make setup-hooks  # Install quality gate pre-commit hooks
make coverage     # Generate coverage report
make check        # Run complete quality pipeline
```

The binary will be installed to `~/.cargo/bin/mp4converter` (via cargo) or `~/.local/bin/mp4converter` (via make). Ensure the appropriate directory is in your PATH.

## 🚀 Quick Start

### Basic Compliance Check
```bash
# Check if your videos meet content delivery standards
mp4converter --dir ~/Videos --compliance

# Example output:
# ✅ Compliant Files: 8 (80%)
# ❌ Non-Compliant Files: 2 (20%)
# 📊 Average Compliance Score: 92/100
```

### Fix Non-Compliant Videos
```bash
# Automatically fix videos to meet standards (PCM audio, H.264 video)
mp4converter --dir ~/Videos --compliance --convert

# Files are saved to ~/Videos/H264/ with original names
# Original files are never modified
```

## 📚 Tutorials & Examples

### Tutorial 1: Professional Content Delivery Preparation

**Scenario**: You have a folder of tutorial videos that need to meet professional content delivery standards for online platforms.

```bash
# Step 1: Analyze your videos
mp4converter --dir ~/my-course-videos --compliance

# Step 2: Review the compliance report
# Look for:
# - Audio codec issues (should be PCM for maximum quality)
# - Resolution problems (should be 1080p or 720p)
# - Frame rate issues (should be standard rates like 30fps)

# Step 3: Fix all non-compliant videos
mp4converter --dir ~/my-course-videos --compliance --convert --verbose

# Result: All videos now have:
# ✅ H.264 video codec (universally compatible)
# ✅ PCM audio (maximum quality, DaVinci Resolve compatible)
# ✅ Standard resolutions (1080p/720p)
# ✅ Proper frame rates
```

### Tutorial 2: DaVinci Resolve Linux Workflow

**Scenario**: Preparing videos for editing in DaVinci Resolve on Linux (which requires PCM audio).

```bash
# Problem: DaVinci Resolve on Linux doesn't work well with AAC audio
# Solution: Convert to PCM audio format

# Step 1: Check current audio codecs
mp4converter --dir ~/raw-footage --compliance | grep -A5 "Audio Codec"

# Step 2: Convert all videos to use PCM audio
mp4converter --dir ~/raw-footage --convert --verbose

# Step 3: Verify conversion
mp4converter --dir ~/raw-footage/H264 --compliance

# You should see:
# Audio Codec Distribution:
#   • 12 videos using pcm_s24le  (24-bit PCM - perfect for DaVinci!)
```

### Tutorial 3: Batch Processing for Content Creators

**Scenario**: You're a content creator with videos from different sources (OBS, screen recordings, phone videos).

```bash
# Process an entire project directory
cd ~/content/youtube-project

# Step 1: Get a detailed analysis
mp4converter --dir . --compliance --verbose

# Step 2: Convert only non-compliant files
mp4converter --dir . --compliance --convert

# The tool will:
# - Skip already compliant files (saves time)
# - Fix audio to PCM for maximum quality
# - Standardize resolutions
# - Preserve original files
# - Keep original filenames in H264 folder
```

### Tutorial 4: Quality Assurance Pipeline

**Scenario**: Ensure all videos in a production pipeline meet quality standards.

```bash
# Create a simple QA script
cat > check_videos.sh << 'EOF'
#!/bin/bash
echo "🎬 Video QA Pipeline"
echo "===================="

# Check compliance
mp4converter --dir ./uploads --compliance > compliance_report.txt

# Extract score
SCORE=$(grep "Average Compliance Score" compliance_report.txt | grep -o '[0-9]*')

# Fail if score is below 90
if [ "$SCORE" -lt "90" ]; then
    echo "❌ Videos don't meet quality standards (Score: $SCORE/100)"
    echo "🔧 Attempting automatic fixes..."
    mp4converter --dir ./uploads --convert --compliance
    exit 1
else
    echo "✅ All videos meet quality standards (Score: $SCORE/100)"
fi
EOF

chmod +x check_videos.sh
./check_videos.sh
```

## 🎮 Advanced Usage

### 📁 **Local Directory Analysis**
```bash
# Analyze local videos for compliance
mp4converter --dir /path/to/videos --compliance

# Convert non-compliant videos with detailed progress
mp4converter --dir /path/to/videos --convert --verbose

# Comprehensive compliance analysis with fixing
mp4converter --dir /path/to/videos --compliance --convert --verbose
```

### ☁️ **Google Drive Integration**
```bash
# Analyze Google Drive videos for compliance
mp4converter --drive --compliance

# Use custom credentials file
mp4converter --drive --compliance --credentials /path/to/creds.json

# Verbose analysis with detailed progress
mp4converter --drive --compliance --verbose

# Future: Fix non-compliant Drive files (coming in Sprint 3)
mp4converter --drive --compliance --convert
```

## 🎯 Common Use Cases

### For Video Editors
```bash
# Fix OBS recordings for DaVinci Resolve
mp4converter --dir ~/obs-recordings --convert --compliance

# Batch process screen captures
mp4converter --dir ~/screen-captures --compliance --convert --verbose
```

### For Course Creators
```bash
# Ensure all course videos meet platform standards
mp4converter --dir ~/course/module-1 --compliance

# Fix and verify all modules
for dir in ~/course/module-*; do
    mp4converter --dir "$dir" --compliance --convert
done
```

### For Content Teams
```bash
# Generate compliance report for stakeholders
mp4converter --dir ./final-videos --compliance > report.txt

# Process videos and keep detailed logs
mp4converter --dir ./raw-videos --compliance --convert --verbose 2>&1 | tee conversion.log
```

## 📖 Understanding Compliance Scores

The compliance scoring system (0-100) evaluates videos based on:

| Score Range | Status | Meaning |
|------------|--------|---------|
| 90-100 | ✅ COMPLIANT | Ready for professional delivery |
| 70-89 | 🟡 MOSTLY COMPLIANT | Minor issues, may work but not optimal |
| 50-69 | 🟠 PARTIALLY COMPLIANT | Significant issues, needs fixes |
| 0-49 | 🔴 NON-COMPLIANT | Major issues, requires conversion |

### Violation Severity Levels
- **🔴 Critical**: Must be fixed (wrong codec, unsupported format)
- **🟡 Warning**: Should be fixed (non-standard resolution, frame rate)
- **🔵 Info**: Nice to fix (audio codec preference, optimization)

## 🔍 Command Reference

### Command-line Options
| Option | Description |
|--------|-------------|
| `-d, --dir <PATH>` | Directory containing video files |
| `-c, --convert` | Enable conversion mode (fixes non-compliant videos) |
| `-v, --verbose` | Show detailed progress and operations |
| `--compliance` | Perform standards compliance analysis |
| `--drive` | Enable Google Drive integration |
| `--credentials <PATH>` | Path to Google Drive credentials JSON |

### File Support
- **Input Formats**: MP4, AVI, MOV (automatically detected)
- **Output Format**: Standards-compliant MP4 with H.264 video and PCM/AAC audio
- **Naming**: Original filenames preserved in H264/ directory
- **Cloud Formats**: Full Google Drive video file support

## ⚙️ Compliance & Conversion Settings

Optimized for **professional content delivery standards** with **maximum quality** and **hardware acceleration**:

### Video Encoding
- **Codec**: H.264 with NVIDIA NVENC hardware acceleration
- **Preset**: p7 (highest quality preset available)
- **Quality Mode**: Constant Quality (CQ) 18 for near-lossless output
- **Profile**: High Profile for maximum feature support
- **Pixel Format**: YUV420p (universal compatibility)

### Audio Encoding  
- **Primary Codec**: PCM (pcm_s24le) - Preferred for maximum quality and DaVinci Resolve compatibility
- **Fallback Codec**: AAC (Advanced Audio Coding) - When PCM is not suitable
- **Sample Rate**: 48kHz for professional quality
- **Bit Depth**: 24-bit for PCM audio
- **Channels**: Stereo (2 channels)

### Compliance Standards
- **Video Codec**: H.264 (AVC) with High Profile
- **Supported Resolutions**: 1920x1080, 1280x720, 854x480 (automatic scaling)
- **Frame Rates**: 23.976, 24, 25, 29.97, 30, 50, 60 fps
- **Audio**: PCM (preferred), ALAC, or AAC with 48kHz sample rate
- **Container**: MP4 with proper metadata structure
- **File Handling**: Non-destructive processing (originals always preserved)
- **Filename Preservation**: Converted files maintain original names

## 🛠️ Development

### Available Make Targets
```bash
make help          # Show all available commands
make test          # Run test suite (38 tests)
make lint          # Run formatting and clippy checks
make coverage      # Generate HTML coverage report  
make check         # Run complete quality pipeline
make setup-hooks   # Install pre-commit quality gates
```

### Quality Gates
Every commit is automatically validated against:
- **Formatting**: Perfect rustfmt compliance
- **Linting**: Zero clippy warnings
- **Testing**: All 38 tests must pass
- **Coverage**: Maintains >80% line coverage
- **TDG Score**: Keeps A-grade (≥90/100)
- **Security**: Zero vulnerabilities
- **Technical Debt**: No TODO/FIXME markers

### Testing
Comprehensive test suite with 81.76% coverage:
- **27 unit tests** covering compliance analysis, metadata extraction, and core functionality
- **11 integration tests** for CLI, argument parsing, and Google Drive integration
- **Error handling tests** for all failure scenarios including cloud operations
- **Property-based tests** for edge cases and compliance scoring
- **Mock tests** for Google Drive API integration

## 🚦 CI/CD Integration

### AWS CodeBuild
Uses `buildspec.yml` for automated building:
- **Environment**: NVIDIA GPU support
- **Dependencies**: Rust, FFmpeg, CUDA toolkit
- **Pipeline**: Format → Lint → Test → Build → Deploy

### Pre-commit Hooks
Automatic quality enforcement via `.git/hooks/pre-commit`:
- Blocks commits that don't meet quality standards
- Provides clear feedback on failures
- Maintains A-grade code quality automatically

## 🔧 Troubleshooting

### Common Issues & Solutions

#### Installation Issues

**Command not found after installation**
```bash
# If installed via cargo, add to PATH:
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# If installed via make, add to PATH:
export PATH="$HOME/.local/bin:$PATH"
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

#### Conversion Issues

**"NVIDIA GPU not detected" or conversion fails**
```bash
# The tool will fall back to CPU encoding if GPU is not available
# To check if NVIDIA GPU is available:
nvidia-smi

# If you don't have NVIDIA GPU, conversions will still work but slower
# The tool automatically detects and uses available hardware
```

**"FFmpeg error" during conversion**
```bash
# Ensure FFmpeg is installed:
sudo apt update && sudo apt install ffmpeg  # Ubuntu/Debian
brew install ffmpeg                          # macOS

# Verify FFmpeg installation:
ffmpeg -version
```

**"No videos found" error**
```bash
# Check supported formats (MP4, AVI, MOV):
ls -la *.{mp4,avi,mov} 2>/dev/null

# Note: The tool is case-sensitive for extensions
# Rename if needed:
for f in *.MP4; do mv "$f" "${f%.MP4}.mp4"; done
```

#### Audio Issues

**DaVinci Resolve still can't read audio**
```bash
# Verify the audio is PCM:
ffprobe -v error -select_streams a:0 -show_entries stream=codec_name your_video.mp4

# Should show: codec_name=pcm_s24le
# If not, re-run conversion:
mp4converter --dir . --convert --compliance
```

**Audio sync issues after conversion**
```bash
# This is rare but can happen with variable frame rate videos
# Solution: Ensure constant frame rate during recording
# OBS Settings: Settings → Video → FPS: 30 (not VFR)
```

#### Performance Issues

**Conversion is very slow**
```bash
# Without GPU acceleration, conversion uses CPU
# Tips to speed up:
# 1. Close other applications
# 2. Process fewer files at once
# 3. Consider upgrading to a system with NVIDIA GPU

# Check system resources:
htop  # or top
```

**Out of disk space during conversion**
```bash
# Check available space:
df -h .

# Converted files can be large (PCM audio is uncompressed)
# Rule of thumb: Need 2x the size of original videos
# Solution: Free up space or use external drive
```

### FAQ

**Q: Why PCM audio instead of AAC?**
A: PCM provides lossless audio quality and is required for DaVinci Resolve on Linux. It's the professional standard for content delivery.

**Q: Can I use this without an NVIDIA GPU?**
A: Yes! The tool will automatically use CPU encoding if GPU is not available. It's slower but works perfectly.

**Q: Will this reduce my video quality?**
A: No, the tool uses high-quality encoding settings (CQ 18) that maintain near-lossless quality.

**Q: Why are my converted files larger?**
A: PCM audio is uncompressed, making files larger but maintaining perfect audio quality. This is ideal for editing and professional delivery.

## 📊 Performance

### Benchmarks
- **Hardware**: RTX 3080, 32GB RAM, NVMe SSD
- **Input**: 1080p H.264 video files
- **Performance**: ~300 FPS encoding speed (real-time playback: 30 FPS)
- **Quality**: Near-lossless output with 30-50% size reduction

### Scaling
- **Batch Processing**: Handles hundreds of files efficiently
- **Memory Usage**: Low memory footprint (~100MB base)
- **GPU Utilization**: Maximizes NVENC encoder usage

## 🤝 Contributing

This project maintains A-grade quality standards. All contributions must:
1. **Pass pre-commit hooks** (automatic quality validation)
2. **Maintain test coverage** above 80%
3. **Keep TDG score** above 90/100 (A grade)
4. **Add tests** for new functionality
5. **Follow Rust best practices** and project conventions

## 📄 License

This project is licensed under the terms specified in the LICENSE file.

## 🚀 Uninstall

```bash
make uninstall
```

---

**Built with ❤️ using Rust and NVIDIA CUDA for maximum performance and reliability.**
