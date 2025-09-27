use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::*;
use humansize::{format_size, DECIMAL};
use image::{ImageFormat, GenericImageView};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
    fn extension(&self) -> &str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::Webp => "webp",
        }
    }
}

#[derive(Debug)]
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
    let stats = Arc::new(Mutex::new(CompressionStats::new()));
    let pb_arc = Arc::new(pb);

    // Process files in parallel for better performance
    files.par_iter().for_each(|file_path| {
        pb_arc.set_message(format!(
            "Processing {}",
            file_path.file_name().unwrap().to_string_lossy()
        ));

        match compress_image(file_path, &output_dir, &args) {
            Ok((original_size, compressed_size)) => {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.files_processed += 1;
                stats_guard.original_size += original_size;
                stats_guard.compressed_size += compressed_size;
            }
            Err(e) => {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.errors.push(format!("{}: {}", file_path.display(), e));
            }
        }

        pb_arc.inc(1);
    });

    pb_arc.finish_with_message("Compression complete!");
    let final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();
    print_results(&final_stats);

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

    // Only resize if dimensions are specified by user
    let img = match (args.max_width, args.max_height) {
        (Some(max_w), Some(max_h)) => {
            img.resize(max_w, max_h, image::imageops::FilterType::Triangle)
        }
        (Some(max_w), None) => {
            let (width, height) = img.dimensions();
            if width > max_w {
                let aspect_ratio = height as f32 / width as f32;
                let new_height = (max_w as f32 * aspect_ratio) as u32;
                img.resize(max_w, new_height, image::imageops::FilterType::Triangle)
            } else {
                img
            }
        }
        (None, Some(max_h)) => {
            let (width, height) = img.dimensions();
            if height > max_h {
                let aspect_ratio = width as f32 / height as f32;
                let new_width = (max_h as f32 * aspect_ratio) as u32;
                img.resize(new_width, max_h, image::imageops::FilterType::Triangle)
            } else {
                img
            }
        }
        (None, None) => img,
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
            let writer = BufWriter::with_capacity(64 * 1024, output_file); // 64KB buffer
            let encoder = JpegEncoder::new_with_quality(writer, args.quality);
            img.write_with_encoder(encoder)?;
        }
        OutputFormat::Png => {
            use image::codecs::png::PngEncoder;
            use std::io::BufWriter;
            
            let output_file = fs::File::create(&output_path)?;
            let writer = BufWriter::with_capacity(64 * 1024, output_file); // 64KB buffer
            let encoder = PngEncoder::new(writer);
            img.write_with_encoder(encoder)?;
        }
        OutputFormat::Webp => {
            // Convert to RGB8 and use webp crate for proper quality control
            let rgb_img = img.to_rgb8();
            let (width, height) = rgb_img.dimensions();
            
            let webp_data = if args.quality >= 100 {
                webp::Encoder::from_rgb(&rgb_img, width, height).encode_lossless()
            } else {
                webp::Encoder::from_rgb(&rgb_img, width, height).encode(args.quality as f32)
            };
            
            std::fs::write(&output_path, &*webp_data)?;
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
    let savings_text = if savings > 0.0 {
        format!("{:.1}% saved", savings)
    } else if savings < 0.0 {
        format!("{:.1}% larger", -savings)
    } else {
        "No change".to_string()
    };
    let colored_savings = if savings > 0.0 {
        savings_text.green().bold()
    } else if savings < 0.0 {
        savings_text.red().bold()
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
