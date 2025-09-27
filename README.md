# 🎨 Image Compressor

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A fast, efficient image compression tool built in Rust. Reduce file sizes while maintaining visual quality with support for JPEG, PNG, and WebP formats.

## ✨ Features

- **High Performance** - Native Rust performance with efficient memory usage
- **Multiple Formats** - Support for JPEG, PNG, and WebP output formats
- **Batch Processing** - Process single files or entire directories recursively
- **Smart Resizing** - Maintain aspect ratios with optional dimension constraints
- **Progress Tracking** - Real-time progress bars with detailed statistics
- **Error Resilience** - Graceful error handling with informative messages

## 🚀 Installation

### From Source
```bash
git clone https://github.com/yourusername/image-compressor.git
cd image-compressor
cargo build --release
```

### Using Cargo
```bash
cargo install image-compressor
```

## 📖 Usage

### Basic Examples

Compress a single image:
```bash
compress image.jpg
```

Compress all images in a directory:
```bash
compress images/ --recursive
```

Convert to WebP with custom quality:
```bash
compress photos/ --format webp --quality 85 --output webp_images/
```

Resize and compress for web:
```bash
compress large_images/ --max-width 1920 --max-height 1080 --recursive
```

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

## 🔧 Requirements

- Rust 1.70 or later
- Supported platforms: Windows, macOS, Linux

## 📊 Performance

- **Fast**: Processes hundreds of images per minute
- **Memory Efficient**: Streaming operations minimize RAM usage
- **Quality Preservation**: Advanced algorithms maintain visual fidelity
- **Space Savings**: Typically achieves 30-70% size reduction

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.