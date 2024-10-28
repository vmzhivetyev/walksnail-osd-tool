
## Swapping bundled ffmpeg version

If you want to swap out the bundled ffmpeg version for any reason, you can do it. You can also just delete ffmpeg and ffprobe binaries from the paths mentioned below, then walksnail-osd-tool should use your system-wide-available ffmpeg.

Just keep in mind that "newer" is not always "better". Different version of ffmpeg have their own quirks, the encoding performance may vary from "this one is not working at all" to "oh gosh this one is much faster!".

| OS | download ffmpeg binaries | bundled ffmpeg location |
| -- | --------------- | -------- |
| windows | https://github.com/GyanD/codexffmpeg/releases | `C:\Program Files\Walksnail OSD Tool\ffmpeg\` |
| macos | https://osxexperts.net | `/Applications/Walksnail OSD Tool.app/Contents/MacOS/` | 
| linux | https://johnvansickle.com/ffmpeg | no binaries are bundled at the moment | 
