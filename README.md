# FileForge

![FileForge Banner](site/public/favicon.svg)

**FileForge** is a high-performance desktop application for fast, safe, and private media optimization. Compress images, convert PNG/JPEG to WebP, clean vector SVG graphics, and transcode video/audio files with zero cloud dependencies.

Built with **Tauri v2**, **Rust**, and **React**.

---

## ✨ Features

- ⚡ **Native Performance**: Multi-threaded parallel file processing powered by Rust and Rayon.
- 🔒 **100% Offline & Private**: All file operations happen locally on your computer. No data is sent to external servers.
- 🖼️ **Image Optimization & Conversion**:
  - Convert PNG and JPEG images to WebP format.
  - Lossless SVG optimization (removes XML comments, editor metadata, and redundant whitespace).
  - Configurable JPEG quality and responsive image scaling.
- 🎥 **Video & Audio Transcoding**:
  - Convert AVI, VOB, and raw video files to compressed MP4 format (H.264 & H.265 / HEVC).
  - Extract audio tracks from videos into MP3 files.
  - Convert WAV audio files to compact MP3.
- 🎨 **Modern Interface**: Clean dark/light theme, progress tracking, and instant preview.

---

## 🚀 Download & Installation

Visit the official download hub on GitHub Pages:
👉 **[https://spiriturban.github.io/file-forge/](https://spiriturban.github.io/file-forge/)**

Installers available for:
- **Windows**: `.exe` Setup / `.msi` Installer
- **macOS**: `.dmg` (Universal / Apple Silicon / Intel)
- **Linux**: `.AppImage` / `.deb`

---

## 🛠️ Development & Building

### Prerequisites

- Node.js 20+
- Rust & Cargo
- (Optional) FFmpeg installed on system PATH for video transcoding features.

### Commands

```bash
# Install dependencies
npm install

# Start development app
npm run dev

# Run frontend build
npm run build

# Run Tauri desktop build
npm run tauri build

# Build marketing website
npm run build:site

# Check version consistency across manifests
npm run version:check
```

---

## 👤 Author

**Spirit Urban**
Developer & creator of high-performance software tools, web applications, and services.

- 🌐 **Personal Hub & Services**: [https://spiriturban.github.io/](https://spiriturban.github.io/)
- 🐙 **GitHub Profile**: [https://github.com/SpiritUrban](https://github.com/SpiritUrban)
- 📁 **Repository**: [https://github.com/SpiritUrban/file-forge](https://github.com/SpiritUrban/file-forge)

---

## 📄 License

Distributed under the [MIT License](LICENSE). Copyright © 2026 Spirit Urban.
