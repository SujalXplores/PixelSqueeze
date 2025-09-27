use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::*;
use humansize::{format_size, DECIMAL};
use image::ImageFormat;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "compress",
    about = "🎨 Beautiful Image Compressor - Reduce file sizes while maintaining quality",
    long_about = "A fast, beautiful image compression tool that supports JPEG, PNG, and WebP formats.\nCompress single files or entire directories with elegant progress tracking."
)]
struct Args {
    #[arg(help = "Input file or directory path")]
    input: PathBuf,

    #[arg(short, long, help = "Output directory (default: ./compressed)")]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "80",
        help = "Compression quality (1-100)"
    )]
    quality: u8,

    #[arg(short, long, default_value = "jpeg", help = "Output format")]
    format: OutputFormat,

    #[arg(short, long, help = "Recursive directory processing")]
    recursive: bool,

    #[arg(long, help = "Maximum width for resizing")]
    max_width: Option<u32>,

    #[arg(long, help = "Maximum height for resizing")]
    max_height: Option<u32>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Jpeg,
    Png,
    Webp,
}

impl OutputFormat {
    fn to_image_format(&self) -> ImageFormat {
        match self {
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::Webp => ImageFormat::WebP,
        }
    }

    fn extension(&self) -> &str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::Webp => "webp",
        }
    }
}

struct CompressionStats {
    files_processed: usize,
    original_size: u64,
    compressed_size: u64,
    errors: Vec<String>,
}

impl CompressionStats {
    fn new() -> Self {
        Self {
            files_processed: 0,
            original_size: 0,
            compressed_size: 0,
            errors: Vec::new(),
        }
    }

    fn savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            let savings = self.original_size.saturating_sub(self.compressed_size) as f64;
            (savings / self.original_size as f64) * 100.0
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.quality == 0 || args.quality > 100 {
        anyhow::bail!("Quality must be between 1 and 100");
    }

    print_banner();

    let output_dir = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("compressed"));
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    let files = collect_image_files(&args.input, args.recursive)?;

    if files.is_empty() {
        println!("{}", "No image files found!".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} image files",
        "📁".blue(),
        files.len().to_string().green().bold()
    );

    let pb = create_progress_bar(files.len());
    let mut stats = CompressionStats::new();

    for file_path in files {
        pb.set_message(format!(
            "Processing {}",
            file_path.file_name().unwrap().to_string_lossy()
        ));

        match compress_image(&file_path, &output_dir, &args) {
            Ok((original_size, compressed_size)) => {
                stats.files_processed += 1;
                stats.original_size += original_size;
                stats.compressed_size += compressed_size;
            }
            Err(e) => {
                stats.errors.push(format!("{}: {}", file_path.display(), e));
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Compression complete!");
    print_results(&stats);

    Ok(())
}

fn print_banner() {
    println!("{}", "╔═════════════════════════════════════╗".cyan());
    println!("{}", "║        🎨 Image Compressor 🎨       ".cyan());
    println!("{}", "║     Fast • Beautiful • Efficient   ".cyan());
    println!("{}", "╚═════════════════════════════════════╝".cyan());
    println!();
}

fn collect_image_files(input: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if input.is_file() {
        if is_image_file(input) {
            files.push(input.to_path_buf());
        }
    } else if input.is_dir() {
        let walker = if recursive {
            WalkDir::new(input).into_iter()
        } else {
            WalkDir::new(input).max_depth(1).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && is_image_file(path) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_string_lossy().to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif"
        )
    } else {
        false
    }
}

fn create_progress_bar(len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb
}

fn compress_image(input_path: &Path, output_dir: &Path, args: &Args) -> Result<(u64, u64)> {
    let original_size = fs::metadata(input_path)?.len();

    let img = image::open(input_path)
        .with_context(|| format!("Failed to open image: {}", input_path.display()))?;

    let img = if let (Some(max_w), Some(max_h)) = (args.max_width, args.max_height) {
        img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
    } else if let Some(max_w) = args.max_width {
        let aspect_ratio = img.height() as f32 / img.width() as f32;
        let new_height = (max_w as f32 * aspect_ratio) as u32;
        img.resize_exact(max_w, new_height, image::imageops::FilterType::Lanczos3)
    } else if let Some(max_h) = args.max_height {
        let aspect_ratio = img.width() as f32 / img.height() as f32;
        let new_width = (max_h as f32 * aspect_ratio) as u32;
        img.resize_exact(new_width, max_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let output_filename = format!(
        "{}.{}",
        input_path.file_stem().unwrap().to_string_lossy(),
        args.format.extension()
    );
    let output_path = output_dir.join(output_filename);

    match args.format {
        OutputFormat::Jpeg => {
            use image::codecs::jpeg::JpegEncoder;
            use std::io::BufWriter;
            
            let output_file = fs::File::create(&output_path)?;
            let mut writer = BufWriter::new(output_file);
            let encoder = JpegEncoder::new_with_quality(&mut writer, args.quality);
            img.write_with_encoder(encoder)?;
        }
        OutputFormat::Png => {
            img.save_with_format(&output_path, args.format.to_image_format())?;
        }
        OutputFormat::Webp => {
            img.save_with_format(&output_path, args.format.to_image_format())?;
        }
    }

    let compressed_size = fs::metadata(&output_path)?.len();
    Ok((original_size, compressed_size))
}

fn print_results(stats: &CompressionStats) {
    println!();
    println!("{}", "╔══════════════ RESULTS ══════════════╗".green());

    let files_text = stats.files_processed.to_string();
    let original_text = format_size(stats.original_size, DECIMAL);
    let compressed_text = format_size(stats.compressed_size, DECIMAL);

    println!(
        "║ {} Files processed: {:<17}",
        "📊".blue(),
        files_text.green().bold()
    );
    println!(
        "║ {} Original size: {:<19}",
        "📏".blue(),
        original_text.green()
    );
    println!(
        "║ {} Compressed size: {:<17}",
        "🗜️ ".blue(),
        compressed_text.green()
    );

    let savings = stats.savings_percent();
    let savings_text = if savings >= 0.0 {
        format!("{:.1}% saved", savings)
    } else {
        format!("{:.1}% larger", -savings)
    };
    let colored_savings = if savings >= 0.0 {
        savings_text.green().bold()
    } else {
        savings_text.yellow().bold()
    };

    println!("║ {} Space change: {:<18}", "💾".blue(), colored_savings);
    println!("{}", "╚═════════════════════════════════════╝".green());

    if !stats.errors.is_empty() {
        println!();
        println!("{} Errors encountered:", "⚠️".yellow());
        for error in &stats.errors {
            println!("  {} {}", "•".red(), error.red());
        }
    }

    println!();
    println!(
        "{} {}",
        "✨".green(),
        "Compression completed successfully!".green().bold()
    );
}
