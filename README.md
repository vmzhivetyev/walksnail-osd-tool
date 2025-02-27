# Fork Enhancements

> [!TIP]
> If you know you have some hardware encoder available (e.g. `hevc_qsv`) but you don't see it in the app, then you most probably need to install a **full build** of ffmpeg and/or another version of ffmpeg ([how to](FFMPEG.md)).

## Key Improvements

* macOS Compatibility: Videos encoded with `hevc_videotoolbox` on macos are now playable in QuickTime and in Finder's Quick Look.
* Aspect Ratio Correction: Added option to rescale input video to 4:3 aspect, addressing distortion issues for videos recorded with "4:3 FULL" setting in VRX.
* FPS and Timing Fixes: Fully fixed issues with final video's fps and OSD-to-video timing desyncs.
* Last used render settings (Encoder, Bitrate, etc.) are now preserved across app relaunches.
* Linux builds in releases! ffmpeg not included.

* OSD Rendering Enhancements:
    * Fixed positioning of OSD glyphs, no more gaps and overlaps.
    * Fixed parsing of [Debug SRT files](https://walksnail.wiki/en/Debug).
    * Loading an SRT file is now optional. You can also toggle SRT rendering with a checkbox.
    * Added checkboxes for Debug SRT components, only enable the things you want.
    * Implemented automatic text wrapping for Debug SRT content.
    * Added checkbox to fully disable OSD rendering in the final video.

* UI Improvements:
    * Dark Mode set as default.
    * Light/Dark Mode preference is now saved between sessions.

* Audio:
    * Original audio stream from input video is now maintained in the output video.

* Encoders:
    * Added Apple ProRes Encoder, it supports true transparent background.
    * Much faster encoding speeds thanks to optimized encoder parameters and CPU bottlenecks optimizations.
    * Support for automatic bitrate selection.

* Other PRs:
    * [Fixed final video to have the same time scale as original](https://github.com/avsaase/walksnail-osd-tool/pull/47)
    * [Added additional OSD rendering settings](https://github.com/avsaase/walksnail-osd-tool/pull/46)

* For Developers:
    * Fixed Debugging: Fixed build profiles settings, disabled optimizations for Debug profile.
    * Simpler Build: Added shell scripts for macOS and Linux to streamline building from source.
 
___

# Original README

<p align="center">
<img width="256" height="176" src="https://user-images.githubusercontent.com/880421/224411816-c0cf1331-c856-42e9-a3d6-1c23b7da7886.png">
</p>
<h1 align="center">Walksnail OSD Tool</h1>

[![Latest release](https://img.shields.io/github/v/release/avsaase/walksnail-osd-tool?include_prereleases&label=latest%20release)](https://github.com/avsaase/walksnail-osd-tool/releases/latest)
[![Latest build](https://img.shields.io/github/last-commit/avsaase/walksnail-osd-tool/master?label=latest%20build)](https://nightly.link/avsaase/walksnail-osd-tool/workflows/release.yaml/master/walksnail-osd-tool-all-platforms.zip)
[![Totally awesome](https://img.shields.io/badge/totally%20awesome-true-blue)](https://github.com/avsaase/walksnail-osd-tool)

Cross-platform tool for rendering the flight controller OSD and SRT data from the Walksnail Avatar HD FPV system on top of the goggle or VRX recording.

![image](https://user-images.githubusercontent.com/880421/228286034-ffd7bf0d-4bb0-4774-9ee1-dd408bd97a88.png)


## Features
- Easy to use graphical user interface.
- Native installer for Windows, App bundle for MacOS.
- Hardware-accelerated encoding powered by ffmpeg.
- Choose between H.264 and H.265 codecs (more can be added later).
- View basic information about the video, OSD, SRT and font files.
- Preview OSD frames before rendering.
- Automatically center the OSD or position it manually.
- Render selected info from the SRT file.
- Selectable output video bitrate (more encoder settings will be added later).
- Upscale output video to 1440p for higher quality when uploading to YouTube.
- Mask OSD items ([demo](https://imgur.com/u8xi2tX)).

Anything else? Open a feature request [here](https://github.com/avsaase/walksnail-osd-tool/issues/new?assignees=&labels=enhancement&template=feature_request.yaml).

## Installation

### Windows
Download and run the installer from the [latest release](https://github.com/avsaase/walksnail-osd-tool/releases/latest).

### MacOS
Download the app bundle for your processor architecture from the [latest release](https://github.com/avsaase/walksnail-osd-tool/releases/latest) and drag it to your Applications folder.

<details>
<summary>Instructions for running the first time</summary>
    
The MacOS binaries provided by this project are not signed with a "Developer ID Certificate". When you try to run the app for the first time you may get a warning from MacOS that it may be malicious software, the developer cannot be verified, it may be damaged, etc. Close the warning, go to System Settings -> Privacy & Security and click "Open Anyway". This should only be required the first time you open the app.

If you think this is annoying you can donate some money [here](https://www.buymeacoffee.com/avsaase) so I can pay Apple for a developer account.
</details>

### Linux
The project builds on Ubuntu in CI but I don't know enough about packaging for Linux to make release binaries. For now you need to build from source.

### Building from source
1. Install the [Rust toolchain](https://www.rust-lang.org/tools/install).
2. Run `cargo install --git https://github.com/avsaase/walksnail-osd-tool.git`. The executable will be installed in `$HOME/.cargo/bin/` and added to your path.
3. To run the app you need the `ffmpeg` and `ffprobe` binaries in your `path` or placed next to the executable you just build.
4. Run the app with `walksnail-osd-tool`.

### Similar projects
- [kirek007/ws-osd-py](https://github.com/kirek007/ws-osd-py): Python-based tool with GUI and CLI. No longer maintained in favor of this project but has a few features that this project currently lacks. Depending on your OS it can require some manual setup due to Python dependencies.
- [shellixyz/hd_fpv_video_tool](https://github.com/shellixyz/hd_fpv_video_tool): Rust-based CLI tool with support for with Walksnail and DJI. Mainly targets Linux and can be difficult to build from source on Windows and MacOS. Has some cool features like live playback of the DVR with OSD without rendering.

## Disclaimer
This project is not affiliated with Walksnail/Caddx.
