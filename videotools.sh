#!/bin/bash

# Video Processing and Analysis Tool
# Purpose: Analyze and convert video files while preserving quality
# Requirements: ffmpeg with NVENC support, ffprobe, jq

# Function to print usage information
print_usage() {
    echo "Usage: $0 [-c|--convert] <directory>"
    echo "Options:"
    echo "  -c, --convert    Convert AVI files to H264 MP4 with quality preservation"
    echo "  -h, --help       Show this help message"
    echo "Without flags, prints detailed metadata for all video files"
    exit 1
}

# Function to format file size in human readable format
format_size() {
    local size=$1
    local units=("B" "KB" "MB" "GB" "TB")
    local unit=0

    while (( size > 1024 && unit < 4 )); do
        size=$(( size / 1024 ))
        ((unit++))
    done

    echo "${size}${units[$unit]}"
}

# Function to analyze video files with detailed metadata
analyze_videos() {
    local dir="$1"
    echo "Analyzing videos in $dir"
    echo "================================="

    find "$dir" -type f \( -name "*.mp4" -o -name "*.avi" \) | while read -r file; do
        echo "File: $(basename "$file")"
        echo "---------------------------------"

        # Get detailed metadata using ffprobe
        ffprobe -v quiet -print_format json -show_format -show_streams "$file" | \
        jq -r '{
            "Video Codec": .streams[0].codec_name,
            "Pixel Format": .streams[0].pix_fmt,
            "Resolution": "\(.streams[0].width)x\(.streams[0].height)",
            "Frame Rate": (.streams[0].r_frame_rate | split("/") | (.[0] / .[1])),
            "Duration": (.format.duration | tonumber | . / 60 | floor | tostring + " minutes " + (. % 1 * 60 | floor | tostring) + " seconds"),
            "Bitrate": ((.format.bit_rate | tonumber) / 1000000 | tostring + " Mbps"),
            "Size": .format.size,
            "Audio Codec": (.streams[] | select(.codec_type=="audio") | .codec_name),
            "Audio Channels": (.streams[] | select(.codec_type=="audio") | .channels),
            "Audio Sample Rate": (.streams[] | select(.codec_type=="audio") | .sample_rate)
        } | to_entries | .[] | "\(.key): \(.value)"'

        # Calculate average bitrate from file size for comparison
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        duration=$(ffprobe -v quiet -select_streams v:0 -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$file")
        avg_bitrate=$(echo "scale=2; $size * 8 / $duration / 1000000" | bc)
        echo "Average Bitrate (calculated): ${avg_bitrate} Mbps"
        echo
    done
}

# Function to convert videos with quality preservation
convert_videos() {
    local dir="$1"
    echo "Converting AVI files to H264 MP4 in $dir"
    echo "================================="

    # Create conversion log
    log_file="conversion_log_$(date +%Y%m%d_%H%M%S).txt"
    echo "Conversion Log - $(date)" > "$log_file"
    echo "=================================" >> "$log_file"

    find "$dir" -type f -name "*.avi" | while read -r file; do
        echo "Processing: $(basename "$file")"
        echo "Processing: $(basename "$file")" >> "$log_file"

        # Get source video details
        source_bitrate=$(ffprobe -v quiet -select_streams v:0 -show_entries stream=bit_rate -of default=noprint_wrappers=1:nokey=1 "$file")
        source_bitrate=${source_bitrate:-8000000} # Default to 8Mbps if unable to detect

        # Calculate target bitrate (match source or minimum 8M)
        target_bitrate=$(( source_bitrate > 8000000 ? source_bitrate : 8000000 ))

        output_file="${file%.avi}.mp4"
        echo "Source bitrate: $(( source_bitrate / 1000000 ))M"
        echo "Target bitrate: $(( target_bitrate / 1000000 ))M"

        # Convert with quality preservation settings
        ffmpeg -hwaccel cuda -hwaccel_output_format cuda -i "$file" \
               -c:v h264_nvenc \
               -preset p7 \
               -rc vbr_hq \
               -cq 18 \
               -b:v "${target_bitrate}k" \
               -maxrate "${target_bitrate}k" \
               -bufsize "$(( target_bitrate * 2 ))k" \
               -tune hq \
               -profile:v high \
               -c:a aac -b:a 320k \
               -movflags +faststart \
               "$output_file" \
               -hide_banner -loglevel warning || {
            echo "Error converting $(basename "$file")" | tee -a "$log_file"
            continue
        }

        # Verify conversion and log results
        echo "Quality comparison:" | tee -a "$log_file"
        {
            echo "Original:"
            ffprobe -v quiet -select_streams v:0 \
                    -show_entries stream=width,height,bit_rate \
                    -of default=noprint_wrappers=1 \
                    "$file"
            echo "Converted:"
            ffprobe -v quiet -select_streams v:0 \
                    -show_entries stream=width,height,bit_rate \
                    -of default=noprint_wrappers=1 \
                    "$output_file"
        } | tee -a "$log_file"

        # Calculate and log file sizes
        original_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
        converted_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file")
        echo "Size comparison:" | tee -a "$log_file"
        echo "Original: $(format_size "$original_size")" | tee -a "$log_file"
        echo "Converted: $(format_size "$converted_size")" | tee -a "$log_file"
        echo "---------------------------------" | tee -a "$log_file"
        echo
    done

    echo "Conversion complete. Log saved to $log_file"
}

# Main script logic
CONVERT=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--convert)
            CONVERT=true
            shift
            ;;
        -h|--help)
            print_usage
            ;;
        *)
            DIR="$1"
            shift
            ;;
    esac
done

# Validate requirements
command -v ffmpeg >/dev/null 2>&1 || { echo "Error: ffmpeg is required but not installed"; exit 1; }
command -v ffprobe >/dev/null 2>&1 || { echo "Error: ffprobe is required but not installed"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed"; exit 1; }

# Check if directory is provided
if [ -z "$DIR" ]; then
    echo "Error: Directory not specified"
    print_usage
fi

# Check if directory exists
if [ ! -d "$DIR" ]; then
    echo "Error: Directory '$DIR' does not exist"
    exit 1
fi

# Check for NVENC support if converting
if [ "$CONVERT" = true ]; then
    if ! ffmpeg -hide_banner -encoders 2>/dev/null | grep -q h264_nvenc; then
        echo "Error: NVENC support not found in ffmpeg"
        exit 1
    fi
fi

# Execute requested operation
if [ "$CONVERT" = true ]; then
    convert_videos "$DIR"
else
    analyze_videos "$DIR"
fi