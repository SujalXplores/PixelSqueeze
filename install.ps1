# PixelSqueeze Installation Script for Windows
# Run with: powershell -ExecutionPolicy Bypass -File install.ps1

Write-Host "🎨 Installing PixelSqueeze..." -ForegroundColor Cyan

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust/Cargo not found. Please install Rust first:" -ForegroundColor Red
    Write-Host "   https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Rust found, installing PixelSqueeze..." -ForegroundColor Green

# Install from crates.io
try {
    cargo install pixelsqueeze
    Write-Host "🎉 PixelSqueeze installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Try it out:" -ForegroundColor Cyan
    Write-Host "  pixelsqueeze --help" -ForegroundColor White
    Write-Host "  pixelsqueeze image.jpg" -ForegroundColor White
    Write-Host "  pixelsqueeze photos/ --recursive" -ForegroundColor White
} catch {
    Write-Host "❌ Installation failed. Try installing manually:" -ForegroundColor Red
    Write-Host "   cargo install pixelsqueeze" -ForegroundColor Yellow
}