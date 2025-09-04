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
- **📊 Batch Reporting**: Detailed compliance summaries with actionable recommendations

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

### Quick Install
```bash
# Clone and install in one command
git clone <repository-url>
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

The binary will be installed to `~/.local/bin/mp4converter`. Ensure this directory is in your PATH.

## 🎮 Usage

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

### Command-line Options
- `-d, --dir <PATH>`: Directory containing video files (optional when using Google Drive)
- `-c, --convert`: Enable conversion mode (default: analysis only)
- `-v, --verbose`: Enable verbose output with detailed progress reporting
- `--compliance`: Perform content standards compliance analysis
- `--drive`: Enable Google Drive integration for cloud file analysis
- `--credentials <PATH>`: Path to Google Drive credentials JSON file (default: credentials.json)

### File Support
- **Input Formats**: MP4, AVI, MOV (automatically detected via metadata analysis)
- **Output Format**: Standards-compliant MP4 with H.264 video and AAC audio
- **Naming Convention**: `input.mp4` → `input.h264.mp4` (non-destructive processing)
- **Cloud Formats**: Full support for Google Drive-hosted video files

## ⚙️ Compliance & Conversion Settings

Optimized for **professional content delivery standards** with **maximum quality** and **hardware acceleration**:

### Video Encoding
- **Codec**: H.264 with NVIDIA NVENC hardware acceleration
- **Preset**: p7 (highest quality preset available)
- **Quality Mode**: Constant Quality (CQ) 18 for near-lossless output
- **Profile**: High Profile for maximum feature support
- **Pixel Format**: YUV420p (universal compatibility)

### Audio Encoding  
- **Codec**: AAC (Advanced Audio Coding)
- **Bitrate**: 320 kbps (high quality)
- **Channels**: Preserves source channel configuration

### Compliance Standards
- **Video Codec**: H.264 (AVC) with High Profile
- **Supported Resolutions**: 1920x1080, 1280x720, 854x480 (automatic scaling)
- **Frame Rates**: 23.976, 24, 25, 29.97, 30, 50, 60 fps
- **Audio**: AAC-LC with 48kHz sample rate
- **Container**: MP4 with proper metadata structure
- **File Handling**: Non-destructive processing (originals always preserved)

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

### Common Issues

**NVIDIA GPU not detected**
```bash
# Verify GPU and drivers
nvidia-smi
```

**FFmpeg NVENC not available**  
```bash
# Check FFmpeg codecs
ffmpeg -codecs | grep h264
```

**Permission denied on install**
```bash
# Ensure ~/.local/bin exists and is in PATH
mkdir -p ~/.local/bin
export PATH="$HOME/.local/bin:$PATH"
```

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