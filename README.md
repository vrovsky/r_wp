# Rust Video Wallpaper

A lightweight, stealthy Rust application that runs an MP4 video as your Windows desktop wallpaper.

## Quick Start

1. **Setup**: Place a video file named `video.mp4` in the project root.
2. **Build & Run**:
   ```bash
   cargo run --release
   ```

## Production
Build the optimized executable:
```bash
cargo build --release
```
The application will run in **Stealth Mode** (no console window). To stop it, use **Task Manager** to end `r_wp.exe`.

## Run on Startup
1. Press `Win + R`, type `shell:startup`, and press Enter.
2. Create a shortcut to `target/release/r_wp.exe` in that folder.
3. Ensure `video.mp4` stays in the same folder as the `.exe`.
