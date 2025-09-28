# Publishing PixelSqueeze CLI

This guide will help you publish PixelSqueeze to make it installable as a global CLI tool.

## Prerequisites

1. **Rust and Cargo** - Already installed ✅
2. **crates.io Account** - Create at https://crates.io/
3. **API Token** - Generate from your crates.io account settings

## Publishing Steps

### 1. Get Your API Token
1. Go to https://crates.io/
2. Sign in with GitHub
3. Go to Account Settings → API Tokens
4. Generate a new token
5. Run: `cargo login <your-token>`

### 2. Verify Package Configuration
Your `Cargo.toml` is already configured with:
- ✅ Proper metadata (name, version, description)
- ✅ License (MIT)
- ✅ Repository URL
- ✅ Keywords and categories
- ✅ Binary configuration

### 3. Test Local Build
```bash
# Build release version
cargo build --release

# Test the executable
./target/release/pixelsqueeze --help
./target/release/pixelsqueeze images/ --quality 80
```

### 4. Publish to crates.io
```bash
# Dry run first (recommended)
cargo publish --dry-run

# If dry run succeeds, publish for real
cargo publish
```

### 5. Verify Installation
After publishing, users can install with:
```bash
cargo install pixelsqueeze
```

Then use globally:
```bash
pixelsqueeze --help
pixelsqueeze photos/ --quality 85 --recursive
```

## Alternative Distribution Methods

### 1. GitHub Releases
Create pre-built binaries for different platforms:

```bash
# Build for current platform
cargo build --release

# The executable will be at:
# Windows: target/release/pixelsqueeze.exe
# macOS/Linux: target/release/pixelsqueeze
```

Upload these to GitHub Releases for direct download.

### 2. Cross-Platform Builds
For multiple platforms, you can use cross-compilation:

```bash
# Install cross-compilation targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu

# Build for different platforms
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

### 3. Package Managers
- **Homebrew** (macOS): Create a formula
- **Chocolatey** (Windows): Create a package
- **Snap** (Linux): Create a snap package

## Post-Publishing

### Update README
Add installation instructions:
```markdown
## Installation

### From crates.io (Recommended)
```bash
cargo install pixelsqueeze
```

### From GitHub Releases
Download the latest binary from [Releases](https://github.com/SujalXplores/PixelSqueeze/releases)
```

### Version Updates
For future updates:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run `cargo publish`

## Troubleshooting

### Common Issues
- **Name already taken**: Choose a different name in `Cargo.toml`
- **Missing metadata**: Ensure all required fields are filled
- **Large package size**: Use `exclude` in `Cargo.toml` to skip unnecessary files

### Package Size Optimization
Your `Cargo.toml` already excludes:
- `images/*` - Sample images
- `compressed/*` - Output directory
- `target/*` - Build artifacts
- `.git/*` - Git history

## Success! 🎉

Once published, your CLI will be available globally via:
```bash
cargo install pixelsqueeze
pixelsqueeze --version
```

Users worldwide can now compress images with a single command!