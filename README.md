# ScrollSnap - High-Performance Scrolling Screenshot Tool

ScrollSnap is a high-performance, cross-platform scrolling screenshot tool built with Tauri v2, Rust, and React. It allows you to manually scroll through any application or webpage, automatically capturing and intelligently stitching the content into a single long image.

## Key Features

- **Flicker-Free Capture**: Uses Windows Native API (`SetWindowDisplayAffinity`) to ensure the recording overlay is invisible to the screenshot engine, providing a seamless, native-like experience without screen flickering.
- **Smart Stitching**: Features a robust, high-tolerance image matching algorithm that eliminates duplicates and handles complex scrolling scenarios with ease.
- **DPI Aware**: Automatically detects and adjusts for high-resolution displays (125%, 150% scaling, etc.), ensuring precise capture area alignment.
- **System-Wide Support**: Works on any desktop application, not just browsers.
- **Privacy Focused**: Runs entirely offline with no data uploaded to the cloud.
- **Modern UI**: Clean, dark-mode interface built with React and Tailwind CSS.

## Tech Stack

- **Framework**: Tauri v2
- **Frontend**: React 18, TypeScript, Tailwind CSS, Zustand, Lucide React
- **Backend**: Rust
  - `xcap`: Cross-platform screen capture (with Windows Graphics Capture support)
  - `windows-rs`: Native Windows API integration
  - `image`: Advanced image processing and stitching
  - `arboard`: System clipboard integration

## Installation

1. Go to the **[Releases](../../releases)** page.
2. Download the installer for your OS:
   - **Windows**: `.exe` or `.msi`
   - **macOS**: `.dmg`

> **Note**: Linux support is currently experimental/paused to focus on Windows/macOS stability.

## Usage Guide

1. **Launch**: Run ScrollSnap.
2. **Start**: Click the **"Start Capture"** button.
3. **Select Area**:
   - The screen will dim.
   - **Click and drag** to draw a green box around the content you want to capture (e.g., a chat window or article).
4. **Scroll**:
   - Release the mouse button. The green box will remain visible.
   - **Manually scroll** the content inside the box at a steady pace.
   - The app will automatically capture and stitch new content as it appears.
5. **Stop**:
   - Press **`Esc`** on your keyboard to stop recording immediately.
   - Alternatively, you can click the stop button (if visible) or wait for the capture to finish.
6. **Save**:
   - A preview window will appear.
   - Click **Save** to download the image as a PNG file.
   - Click **Copy** to copy the image directly to your clipboard.

## Development

If you want to build from source:

### Prerequisites
- **Node.js**: v18+
- **Rust**: v1.75+ (stable)
- **Windows**: Visual Studio C++ Build Tools

### Build Steps

1. **Clone**:
   ```bash
   git clone https://github.com/Goblin-Master/scroll-snap.git
   cd scroll-snap
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Run in Dev Mode**:
   ```bash
   npm run tauri dev
   ```

4. **Build Release**:
   ```bash
   npm run tauri build
   ```

## Troubleshooting

**Q: The screenshot is black or static?**
A: This usually happens on Windows if "Hardware Acceleration" conflicts with the capture API. However, our new native implementation should resolve this. Ensure you are using the latest version (v1.0.0+).

**Q: Scrolling is too fast/blurry?**
A: Scroll at a moderate, steady speed. Extremely fast scrolling might cause the stitching algorithm to miss the overlap.

## License

MIT
