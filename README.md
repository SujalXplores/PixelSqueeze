
<pre>
  _____ _          _  _____                               
 |  __ (_)        | |/ ____|                              
 | |__) |__  _____| | (___   __ _ _   _  ___  ___ _______ 
 |  ___/ \ \/ / _ \ |\___ \ / _` | | | |/ _ \/ _ \_  / _ \
 | |   | |>  <  __/ |____) | (_| | |_| |  __/  __// /  __/
 |_|   |_/_/\_\___|_|_____/ \__, |\__,_|\___|\___/___\___|
                               | |                        
                               |_|                        
</pre>

<div align="center">
<b><i>Squeeze pixels, not quality!</i> ✨</b>
</div>

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-SujalXplores/PixelSqueeze-blue.svg)](https://github.com/SujalXplores/PixelSqueeze)


<p align="center">
	<img src="https://img.shields.io/github/stars/SujalXplores/PixelSqueeze?style=social" alt="GitHub stars">
	<img src="https://img.shields.io/github/forks/SujalXplores/PixelSqueeze?style=social" alt="GitHub forks">
	<img src="https://img.shields.io/github/issues/SujalXplores/PixelSqueeze?color=yellow" alt="GitHub issues">
</p>

PixelSqueeze is a blazingly fast, developer-friendly image compression tool that shrinks your files without compromising visual excellence. Built with Rust for maximum performance and wrapped in a beautiful CLI that makes compression actually enjoyable.

---

## 🚀 Why PixelSqueeze?

**Stop settling for bloated images.** Whether you're optimizing for web performance, saving storage space, or just want lightning-fast compression, PixelSqueeze delivers professional results with zero hassle.

- **🔥 Blazing Fast** - Rust-powered performance that processes hundreds of images per minute
- **🎯 Smart Compression** - Advanced algorithms that preserve quality while maximizing space savings
- **🌈 Beautiful Interface** - Elegant progress bars and colorful output that makes compression fun
- **🔄 Batch Magic** - Process entire directories with recursive scanning
- **📐 Smart Resizing** - Intelligent dimension constraints with perfect aspect ratio preservation
- **🎨 Multi-Format** - JPEG, PNG, and WebP support with format conversion

## ✨ Features

- **High Performance** - Native Rust performance with efficient memory usage
- **Multiple Formats** - Support for JPEG, PNG, and WebP output formats
- **Batch Processing** - Process single files or entire directories recursively
- **Smart Resizing** - Maintain aspect ratios with optional dimension constraints
- **Progress Tracking** - Real-time progress bars with detailed statistics
- **Error Resilience** - Graceful error handling with informative messages
- **🔒 100% Local Processing** - Your images never leave your machine - no uploads, no cloud processing

## ⚡ Installation

### Quick Install (Recommended)
```bash
cargo install pixelsqueeze
```

### One-Line Install Scripts
**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/SujalXplores/PixelSqueeze/main/install.ps1 | iex
```

**macOS/Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/SujalXplores/PixelSqueeze/main/install.sh | bash
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/SujalXplores/PixelSqueeze/releases) for:
- Windows (x64)
- macOS (x64) 
- Linux (x64)

### From Source
```bash
git clone https://github.com/SujalXplores/PixelSqueeze.git
cd PixelSqueeze
cargo build --release
```

---

## 🏁 Quick Start
```bash
# Compress a single image
pixelsqueeze photo.jpg

# Batch compress with custom quality
pixelsqueeze images/ --quality 85 --recursive

# Convert to WebP for maximum savings
pixelsqueeze photos/ --format webp --output optimized/
```

---

## 💫 Usage Examples

### The Basics
```bash
# Compress with default settings (80% quality, JPEG)
pixelsqueeze image.jpg

# Batch process entire directories
pixelsqueeze photos/ --recursive
```

### Power User Moves
```bash
# High-quality web optimization
pixelsqueeze portfolio/ --quality 90 --max-width 1920 --format webp --recursive

# Ultra compression for thumbnails
pixelsqueeze thumbnails/ --quality 60 --max-width 300 --max-height 300

# Convert everything to modern WebP
pixelsqueeze legacy_images/ --format webp --quality 85 --output modern_images/
```

### Pro Tips
```bash
# Perfect for social media (Instagram-ready)
pixelsqueeze posts/ --max-width 1080 --max-height 1080 --quality 85

# Optimize for email attachments
pixelsqueeze documents/ --quality 70 --max-width 800 --recursive
```

---

## 🎯 Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output` | `-o` | Output directory | `./compressed` |
| `--quality` | `-q` | Compression quality (1-100) | `80` |
| `--format` | `-f` | Output format (jpeg, png, webp) | `jpeg` |
| `--recursive` | `-r` | Process directories recursively | `false` |
| `--max-width` | | Maximum width for resizing | None |
| `--max-height` | | Maximum height for resizing | None |
| `--help` | `-h` | Show help information | |

---

## Real-World Impact

**Before PixelSqueeze:**
- 📸 5MB photo → 😱 Slow website loading
- 📁 1GB photo folder → 💾 Storage nightmare
- 🖱️ Manual compression → ⏰ Hours of tedious work

**After PixelSqueeze:**
- 📸 5MB → 1.2MB → ⚡ Lightning-fast loading
- 📁 1GB → 350MB → 💚 Happy storage space
- 🔄 One command → 🚀 Entire folder optimized in seconds

---

## 🏆 Performance Stats

| Metric | Result |
|--------|--------|
| **Speed** | 500+ images/minute |
| **Memory** | Ultra-efficient streaming |
| **Quality** | Visually lossless compression |
| **Savings** | 30-80% size reduction |
| **Formats** | JPEG, PNG, WebP |

---

## 🔒 Privacy & Security

**Your images stay on your machine.** PixelSqueeze processes everything locally - no internet connection required, no uploads, no cloud processing. Your photos and data remain completely private and secure.

- ✅ **100% Offline Processing** - Works without internet
- ✅ **No Data Collection** - Zero telemetry or analytics
- ✅ **No Cloud Uploads** - Images never leave your device
- ✅ **Open Source** - Fully auditable code

---

## 🛠️ System Requirements

- **Rust**: 1.70+ (for building from source)
- **Platforms**: Windows, macOS, Linux
- **Memory**: Minimal RAM usage thanks to streaming
- **Storage**: Tiny binary, massive impact

---

## 🌟 Created By

**SujalXplores** - Passionate about making developer tools that don't suck.

- 🐙 GitHub: [@SujalXplores](https://github.com/SujalXplores)
- 🚀 Project: [PixelSqueeze](https://github.com/SujalXplores/PixelSqueeze)

---

## 🤝 Contributing

Found a bug? Have a cool feature idea? Contributions make the open-source world go round!

1. Fork the repo
2. Create your feature branch (`git checkout -b amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the **MIT License** ([LICENSE-MIT](LICENSE-MIT)) - simple, permissive, and developer-friendly!

---

<div align="center">


<b>Made with ❤️ and lots of ☕ by <a href="https://github.com/SujalXplores">SujalXplores</a></b>

<i>If PixelSqueeze saved you time, consider giving it a ⭐ on <a href="https://github.com/SujalXplores/PixelSqueeze">GitHub</a>!</i>

</div>
