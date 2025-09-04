# MP4 Converter

[![TDG Score](https://img.shields.io/badge/TDG-92.7%2F100%20(A)-brightgreen)](https://github.com/paiml/paiml-mcp-agent-toolkit)
[![Coverage](https://img.shields.io/badge/Coverage-81.76%25-brightgreen)](https://github.com/taiki-e/cargo-llvm-cov)
[![Quality](https://img.shields.io/badge/Quality-A%20Grade-brightgreen)](https://github.com/paiml/paiml-mcp-agent-toolkit)

GPU-accelerated video processing tool built in Rust with NVIDIA CUDA support. Engineered for high performance, reliability, and production-grade quality standards.

## 🚀 Features

- **🎯 GPU Acceleration**: Fast video conversion using NVIDIA NVENC hardware encoding
- **📁 Batch Processing**: Process entire directories with intelligent file detection
- **📊 Progress Tracking**: Real-time progress bars with ETA and detailed analytics
- **🛡️ Non-destructive**: Original files are never modified or overwritten
- **⚡ High Performance**: Optimized settings for maximum quality and speed
- **🔍 Video Analytics**: Comprehensive metadata extraction and reporting

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
# Install with development tools
make setup-hooks  # Install quality gate pre-commit hooks
make coverage     # Generate coverage report
```

The binary will be installed to `~/.local/bin/mp4converter`. Ensure this directory is in your PATH.

## 🎮 Usage

### Basic Commands
```bash
# Analyze videos in directory (dry-run)
mp4converter --dir /path/to/videos

# Convert videos with progress tracking
mp4converter --dir /path/to/videos --convert

# Verbose output with detailed information
mp4converter --dir /path/to/videos --convert --verbose
```

### Command-line Options
- `-d, --dir <PATH>`: Directory containing video files (required)
- `-c, --convert`: Actually convert files (default: analysis only)
- `-v, --verbose`: Enable verbose output with detailed progress

### File Support
- **Input Formats**: MP4, AVI (automatically detected)
- **Output Format**: MP4 with H.264 video and AAC audio
- **Naming Convention**: `input.mp4` → `input.h264.mp4`

## ⚙️ Conversion Settings

Optimized for **maximum quality** and **hardware acceleration**:

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

### Output Characteristics
- **Resolution**: Maintains original source resolution
- **Container**: MP4 for maximum compatibility
- **File Handling**: Non-destructive (originals preserved)

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
- **27 unit tests** covering all core functionality
- **11 integration tests** for CLI and argument parsing
- **Error handling tests** for all failure scenarios
- **Property-based tests** for edge cases

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