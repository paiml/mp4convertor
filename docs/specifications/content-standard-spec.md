### **Content Delivery Video Format Guidelines**

**Overview**  
These are the recommended guidelines for delivering your video content as ready for publication. Please review this document, discuss with your producer, and send in a test movie before exporting or recording your entire course.

**Why do we need high-quality exports?**  
We require high-quality exports to make sure that your course looks its best on the platform. Your course will exist alongside other professional courses and be available to members on multiple platforms. We want to make sure that all courses have a consistent feel from a quality level.

Video and audio mastering are a technical science. We don’t expect you to know the ins and outs of the specifics of codecs. We just want to start our process with the highest quality export available.

**Helpful Hints (for recording content)**

* Maintain consistent video and audio properties throughout the production process.
    * For example, if a screen capture is recorded at 1280x720, you should maintain that resolution from start to finish, including the mastering phase.
* Screen capture should be recorded at a minimum of 1280x720 and maximum of 1920x1080.
    * The minimum recording resolution is 1280x720 for HD and 720x1280 for vertical video
* We strongly recommend recording all footage **at a frame rate suitable for web use, specifically 30fps.** Recording at this rate helps prevent unnecessary processing that could introduce unwanted changes to the visual quality and overall aesthetic of the video.
* These resolutions help increase the legibility of text throughout any interface.
    * High resolutions can make it difficult for UIs and text to scale and may cause legibility issues
* Vertical Video
    * Keep the lower 3rd of the frame clear for platform-provided captions.
    * Don't use running captions (captions that repeat everything you say).
    * Record your video using 'high compatibility' mode found in most phone recording apps.

**Helpful Hints (for exporting content)**

* If you’re unsure if the delivery format works, contact your producer. They will coordinate with our internal team to provide feedback and guidance.
* Please submit a test video to your producer so that format specifications can be verified and approved.
* Please avoid any normalization or filters on the audio.
* Live-action media will always require a higher data rate than screen capture.
* For H.264 mastering, locate and apply the multi-pass setting. This improves the overall quality of the export.
* Always encode to the highest data-rate targets listed in the specifications above.
* Keep the original file on your local hard drive. If necessary, this high-quality source file can be used for re-delivery of videos if something fails during the transfer.

### 

### 

### **Preferred Video and Audio Formats**

**Preferred Resolutions**

| Screen Capture | 1280x720 (720p), 1360x768, 1280x800, 1600x900, 1920x1080 (1080p) |
| :---- | :---- |
| **Mixed Media** | 1280x720 (720p), 1360x768, 1920x1080 (1080p) |
| **Live Action**  | 1280x720 (720p), 1920x1080 (1080p) |
| **Vertical Video** | 720x1280, 1080x1920, 2160x3840 |

**Other Acceptable Resolutions**

| 16x10 Aspect Ratio | 1440x900, 1680x1048 |
| :---- | :---- |
| **16x9 Aspect Ratio** | 1440x810, 1600x900 |
| **9x16 Aspect Ratio (Vertical)** | 720x1280, 1080x1920, 2160x3840 |

**Aspect Ratios**

| HD | 16:9, 16:10, 9:16 (Vertical) |
| :---- | :---- |

**Preferred Frame Rates**  
All frame rates should be captured and delivered at a **constant frame rate**.

| Screen Capture | 15, 29.97, 30 |
| :---- | :---- |
| **Live Action or Mixed Media  or Vertical Video** | 23.976, 24, 25,  29.97, 30 |

**H.264 File Format Specifications**

| Property | Specification |
| ----- | ----- |
| **Codec** | H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10 |
| **Wrapper/Container** | .mov/.mp4/.MOV/.MP4 (.mkv is not supported) |
| **Frame Rate** | Select the same frame rate that was used during recording (24, 25, 30, 60 fps supported) |
| **Profile** | Main, High |
| **Frame Interval** | At least 2 Seconds (keyframe interval) |
| **Aspect Ratio** | Maintain Aspect Ratio (16:9, 16:10, 9:16) |
| **Quality** | 100% |
| **Bit Rate** | 6000–8000 kbps (Screen Capture), 8000–15000 kbps (Live Action) |
| **Scene Change** | Auto |
| **Chroma Subsampling** | 4:2:0 or 4:2:2 for improved color detail |
| **Color Space** | Rec. 709 (SDR) |

**DNxHD File Format Specifications**

| Codec | DNxHD SQ 8 bit |  |
| :---- | :---- | ----- |
| **Wrapper/Container** | .mxf/.mov |  |
| **Aspect Ratio** | Maintain Aspect Ratio |  |
| **Field Order** | Progressive |  |

### **ProRes 422 File Format Specifications**

| Codec | Apple ProRes 422 |
| :---- | :---- |
| **Container** | `.mov` (QuickTime) |
| **Bit Rate** | Default |
| **Chroma Subsampling** | 4:2:2 |
| **Bit Depth** | 10-bit |
| **Color Space** | Rec. 709 (SDR) |

**Preferred Audio Encoding**

| Codec | PCM, ALAC, Uncompressed |
| :---- | :---- |
| **Sampling Rate** | 44.1/48 kHz  |
| **Bit Depth** | 16/24 |
| **Channel Layout** | Stereo L R |
| **AAC\* (Advanced Audio Recording Bit Rates)** | 320 kbps CBR |

**\* Avoid using AAC if possible. Use AAC only if another codec is unachievable. The audio standard of professional learning content is a differentiator in the quality of the experience provided to the viewer. AAC is a compressed format and limits our ability to make your content sound as good as possible.**

### **HDR Content Restrictions**

Our video delivery pipelines do not support HDR content. The following HDR properties should be avoided if possible.

| Property | Unsupported  |
| ----- | ----- |
| **Color Space** | BT.2020, DCI-P3 |
| **Codec** | H.265, HEVC |
| **Dynamic Range** | High Dynamic Range (HDR10, HDR10+, Dolby Vision, HLG) |
| **Bit Depth** | 10-bit, 12-bit |
| **Gamma Curve** | SMPTE ST 2084 (PQ), HLG |
| **Metadata** | Static or dynamic HDR metadata |

