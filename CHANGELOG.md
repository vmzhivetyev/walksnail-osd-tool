# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.3] - 2024-11-17

### Changed
- Loading an SRT file is now optional.
- Added checkbox to toggle SRT rendering.

### Other
- Linux builds in releases! ffmpeg not included.

## [1.1.2] - 2024-10-28

### Changed
- Encoding Speed: Improved encoding performance when using `*_nvenc` encoders.
- Encoding Quality **in Constant Quality Mode**: Changed `qp` values to improve encoded video quality for `hevc_nvenc` and `h264_nvenc`.

## [1.1.1] - 2024-10-28

### Added
- Encoding bitrate: Output video bitrate is now displayed in the status bar during rendering.

### Fixed
- Constant Quality feature: Resolved an issue where some encoders used the fixed bitrate set by the slider instead of the CRF when this feature was enabled.
- App freezes: Fixed freezing of the app when the encoder crashed.
- Encoders crashes: Fixed invalid parameters supplied to `*_nvenc` encoders.

### Changed
- Undetected encoders are no more visible in the app. Detection is done by running an encoder and checking if it crashes, so undetected encoders do not work anyway.
- Hardware decoder will now be automatically used if available, this decreases CPU load.

## [1.1.0] - 2024-10-26

### Added
- "Constant Quality" feature: Prioritizes video quality by asking the encoder for a target quality level instead of bitrate. (1850df3)
- Rendering process live view: Added a feature to render live preview of the output video frames while rendering. This feature doesn't harm encoding speed (yes, at all). (3cb3419)

### Fixed
- Wrong encoder was used: Fixed an issue where the `libx264` encoder was used instead of the selected encoder in the UI, occurring after app updates or on fresh installs. (e9f3a28)

### Changed
- Encoding performance (all encoders): Achieved up to 30% faster encoding by separating *frame generation* and *feeding into ffmpeg* into distinct threads, coupled with a new queue. This change affects all encoders, though performance gains may vary depending on pipeline bottlenecks or full CPU utilization. (3afb2bb)
- Encoding performance on NVIDIA GPUs: Major speed enhancements, up to 60% faster encoding without upscaling and up to 6x speedup with upscaling, specifically for `hevc_nvenc` and `h264_nvenc` encoders. (85e7912)
- OSD glyph caching: Resized OSD glyphs images are now cached at runtime, reducing CPU usage by osd rendering (frame generation) thread. (ebc4ff0)

### Known Issues
- `nvenc` encoder won't work.

## [1.0.0] - 2024-09-24

### Added
* Aspect Ratio Correction: Added option to rescale input video to 4:3 aspect, addressing distortion issues for videos recorded with "4:3 FULL" setting in VRX.
* Added checkboxes for Debug SRT components, only enable the things you want.
* Implemented automatic text wrapping for Debug SRT content.
* Added checkbox to fully disable OSD rendering in the final video.
* Added Apple ProRes Encoder, it supports true transparent background.

### Fixed
* macOS Compatibility: Videos encoded with `hevc_videotoolbox` on macos are now playable in QuickTime and in Finder's Quick Look.
* FPS and Timing Fixes: Fully fixed issues with final video's fps and OSD-to-video timing desyncs.
* Last used render settings (Encoder, Bitrate, etc.) are now preserved across app relaunches.
* Fixed positioning of OSD glyphs, no more gaps and overlaps.
* Fixed parsing of [Debug SRT files](https://walksnail.wiki/en/Debug).
* Light/Dark Mode preference is now saved between sessions.
* Original audio stream from input video is now maintained in the output video.
* Other PRs:
  * [Fixed final video to have the same time scale as original](https://github.com/avsaase/walksnail-osd-tool/pull/47)
  * [Added additional OSD rendering settings](https://github.com/avsaase/walksnail-osd-tool/pull/46)
* For Developers:
  * Fixed Debugging: Fixed build profiles settings, disabled optimizations for Debug profile.
  * Simpler Build: Added shell scripts for macOS and Linux to streamline building from source.

### Changed
* Dark Mode set as default.

## [0.3.0] - 2024-03-23

### Added

- Load last used OSD font file on startup (@dz0ny).
- Option to render video with a chroma key background instead of the input video so the OSD can be overlayed in a video editor.
- Support for Betaflight 4.5 four color fonts.
- Support for INAV two color fonts ([#43](https://github.com/avsaase/walksnail-osd-tool/pull/43), @mmosca).
- Support for 4K and 2.7K DVR ([#43](https://github.com/avsaase/walksnail-osd-tool/pull/43), @mmosca).

### Fixed

- Bug that caused font files with unexpected number of characters to not open.

## [0.2.0] - 2023-04-23

### Added

- Save OSD and SRT options between program runs.
- Custom position and text size of SRT data.
- Option to adjust OSD playback speed to correct for OSD lag with <=32.37.10 firmware.
- Check for app updates during startup.
- Hide/mask OSD elements from the rendered video ([demo](https://i.imgur.com/u8xi2tX.mp4)).
- Tooltips explaining options and settings.

### Changed

- When loading a SRT file with distance data the distance checkbox doesn't get automatically checked.
- Options sections can be collapsed to save screen space.

## [0.1.0] - 2023-03-31

### Fixed

- Parsing of firmware version 32.37.10 SRT data.

## [0.1.0-beta4] - 2023-03-28

### Added

- Render data from the SRT file on the video. Select which values are rendered.
- Automatically load the matching OSD and SRT files when importing a video (they must be in the same folder and have the same file name).
- Upscale output video to 1440p to get better compression on YouTube.

### Changed

- New UI layout with better support for different screen sizes.
- Many small UI tweaks.

### Fixed

- Show correct number of characters in font file.

## [0.1.0-beta3] - 2023-03-21

### Added

- Open files by dropping them on the window.
- Improve render speed.
- Logging of ffmpeg errors and warnings.
- Option to select undetected encoders (use at your own risk).
- Dark theme (default light, toggle by clicking the sun/moon icon in the top right).

### Changed

- Improved handling of ffmpeg events.

### Fixed

- Issue with non-critical ffmpeg errors stopping the render process.
- Output videos not playable in some video players.

## [0.1.0-beta2] - 2023-03-15

### Added

- Make main window resizable in vertical direction to accomodate retina displays and screens with lower resolutions.
- Display errors from ffmpeg.
- Display tooltip when hovering over start render button when it is disabled.

### Changed

- Improved formatting of "About" window.
- Improved display of render status when rendering is finished or cancelled.

### Fixed

- Check for `hevc_videotoolbox` encoder on MacOS.
- Stop ffmpeg decoder when encoder returns error.
- Fixed version info display.
- Properly disable buttons that cannot be used.

## [0.1.0-beta1] - 2023-03-11

### Added

First beta release with limited features.
