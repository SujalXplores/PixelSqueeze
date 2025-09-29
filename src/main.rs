use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use comfy_table::Table;
use humansize::{format_size, DECIMAL};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "pixelsqueeze",
    about = "PixelSqueeze - High-performance image compression",
    long_about = "Lightning-fast image compression that reduces file sizes while maintaining quality.\nSupports JPEG, PNG, and WebP formats with progress tracking and batch processing.",
    version
)]
struct Args {
    #[arg(help = "Input file or directory path")]
    input: PathBuf,

    #[arg(short, long, help = "Output directory (default: ./compressed)")]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "55",
        help = "Compression quality (1-100)"
    )]
    quality: u8,

    #[arg(
        long,
        default_value = "0",
        help = "Minimum compression savings percentage to keep file (0-100)"
    )]
    min_savings: f64,

    #[arg(
        long,
        help = "Keep metadata (EXIF, etc.) in compressed images"
    )]
    keep_metadata: bool,

    #[arg(short, long, default_value = "png", help = "Output format")]
    format: OutputFormat,

    #[arg(short, long, help = "Recursive directory processing")]
    recursive: bool,


}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Jpeg,
    Png,
    Webp,
}

impl OutputFormat {
    const fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }
}

#[derive(Debug, Clone)]
struct FileResult {
    filename: String,
    original_size: u64,
    compressed_size: u64,
}



#[derive(Debug, Default)]
struct CompressionStats {
    files_processed: usize,
    original_size: u64,
    compressed_size: u64,
    errors: Vec<String>,
    file_results: Vec<FileResult>,
}

impl CompressionStats {
    const fn new() -> Self {
        Self {
            files_processed: 0,
            original_size: 0,
            compressed_size: 0,
            errors: Vec::new(),
            file_results: Vec::new(),
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

    fn add_file_result(&mut self, result: FileResult) {
        self.files_processed += 1;
        self.original_size += result.original_size;
        self.compressed_size += result.compressed_size;
        self.file_results.push(result);
    }


}

fn main() -> Result<()> {
    let start_time = Instant::now();
    
    let args = Args::parse();
    validate_args(&args)?;
    
    print_banner();

    let output_dir = args.output.as_deref()
        .map_or_else(|| PathBuf::from("compressed"), PathBuf::from);
    
    fs::create_dir_all(&output_dir).with_context(|| {
        format!("Failed to create output directory: {}", output_dir.display())
    })?;

    let files = collect_image_files(&args.input, args.recursive)?;

    if files.is_empty() {
        print_no_files_found();
        return Ok(());
    }

    print_files_found(files.len());

    let processing_start = Instant::now();
    let stats = process_files_parallel(&files, &output_dir, &args)?;
    let processing_time = processing_start.elapsed();
    
    print_results(&stats, processing_time, start_time.elapsed());

    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if !(1..=100).contains(&args.quality) {
        anyhow::bail!("Quality must be between 1 and 100");
    }
    Ok(())
}

fn print_no_files_found() {
    println!("{}", "No image files found".bright_red());
}

fn print_files_found(count: usize) {
    println!("Found {} images", count.to_string().bright_green());
}

fn process_files_parallel(
    files: &[PathBuf], 
    output_dir: &Path, 
    args: &Args
) -> Result<CompressionStats> {
    let pb = create_progress_bar(files.len());
    let stats = Arc::new(Mutex::new(CompressionStats::new()));
    let pb_arc = Arc::new(pb);

    // Configure rayon for maximum performance
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get().max(1))
        .build_global()
        .unwrap_or_else(|_| {}); // Ignore if already initialized

    // Process files in parallel with optimized chunking for ultra-fast performance
    files.par_iter().for_each(|file_path| {
        let filename = file_path
            .file_name()
            .map_or_else(|| "unknown".to_string(), |n| n.to_string_lossy().to_string());
        
        pb_arc.set_message(filename.clone());

        // Force compression - no skipping allowed
        let result = compress_image_force(file_path, output_dir, args)
            .map(|(original_size, compressed_size)| {
                create_file_result(filename.clone(), original_size, compressed_size)
            });

        match result {
            Ok(file_result) => {
                if let Ok(mut stats_guard) = stats.lock() {
                    stats_guard.add_file_result(file_result);
                }
            }
            Err(e) => {
                if let Ok(mut stats_guard) = stats.lock() {
                    stats_guard.errors.push(format!("{}: {}", filename, e));
                }
            }
        }

        pb_arc.inc(1);
    });

    pb_arc.finish_with_message("Compression complete");
    
    Arc::try_unwrap(stats)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap stats"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to get stats from mutex"))
}

fn create_file_result(
    filename: String,
    original_size: u64,
    compressed_size: u64,
) -> FileResult {
    FileResult {
        filename,
        original_size,
        compressed_size,
    }
}

// New function that forces compression of ALL images - no skipping
fn compress_image_force(input_path: &Path, output_dir: &Path, args: &Args) -> Result<(u64, u64)> {
    let original_size = fs::metadata(input_path)?.len();

    // Load image - always process, never skip
    let img = image::open(input_path)
        .with_context(|| format!("Failed to open image: {}", input_path.display()))?;



    let output_filename = create_output_filename(input_path, &args.format)?;
    let output_path = output_dir.join(output_filename);

    // Smart compression based on input and output formats
    compress_with_smart_settings(&img, &output_path, &args.format, args.quality, input_path)?;

    let compressed_size = fs::metadata(&output_path)?.len();
    
    // If the compressed file is more than 50% larger, use original copy instead
    if compressed_size > original_size + (original_size / 2) {
        // Copy original file instead of the enlarged compressed version
        let original_output = output_dir.join(
            input_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
        );
        fs::copy(input_path, &original_output)?;
        let _ = fs::remove_file(&output_path); // Remove the enlarged version
        Ok((original_size, original_size))
    } else {
        Ok((original_size, compressed_size))
    }
}



fn print_banner() {
    println!("{} {}", "PixelSqueeze".bright_white().bold(), env!("CARGO_PKG_VERSION").bright_green());
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
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        "jpg", "jpeg", "png", "webp", "bmp", "tiff", "gif"
    ];
    
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn create_progress_bar(len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:50} {pos:>3}/{len:3} {msg}")
        .expect("Invalid progress bar template")
        .progress_chars("█▉▊▋▌▍▎▏ ");
    
    pb.set_style(style);
    pb
}



fn create_output_filename(input_path: &Path, format: &OutputFormat) -> Result<String> {
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", input_path.display()))?;
    
    Ok(format!("{}.{}", stem, format.extension()))
}





fn compress_with_smart_settings(
    img: &image::DynamicImage, 
    output_path: &Path, 
    format: &OutputFormat, 
    quality: u8,
    input_path: &Path
) -> Result<()> {
    let input_ext = input_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    match format {
        OutputFormat::Jpeg => {
            // For JPEG output, always compress with specified quality
            compress_jpeg(img, output_path, quality)
        },
        OutputFormat::Png => {
            // PNG compression - avoid converting JPEG to PNG unless necessary
            if input_ext == "jpg" || input_ext == "jpeg" {
                // Converting JPEG to PNG usually increases size, use higher compression
                compress_png_aggressive(img, output_path)
            } else {
                compress_png(img, output_path)
            }
        },
        OutputFormat::Webp => {
            // WebP is generally efficient for all input types
            compress_webp(img, output_path, quality)
        },
    }
}

fn compress_jpeg(img: &image::DynamicImage, output_path: &Path, quality: u8) -> Result<()> {
    use image::codecs::jpeg::JpegEncoder;
    use std::io::BufWriter;
    
    // Convert to RGB to strip alpha channel and metadata
    let rgb_img = img.to_rgb8();
    let output_file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create JPEG file: {}", output_path.display()))?;
    
    // Use buffered writer for better performance
    let buf_writer = BufWriter::new(output_file);
    let encoder = JpegEncoder::new_with_quality(buf_writer, quality);
    rgb_img.write_with_encoder(encoder)
        .with_context(|| "Failed to encode JPEG")?;
    
    Ok(())
}

fn compress_png(img: &image::DynamicImage, output_path: &Path) -> Result<()> {
    use image::codecs::png::{PngEncoder, CompressionType, FilterType};
    use std::io::BufWriter;
    
    let output_file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create PNG file: {}", output_path.display()))?;
    
    // Use buffered writer with proper PNG compression settings
    let buf_writer = BufWriter::new(output_file);
    let encoder = PngEncoder::new_with_quality(
        buf_writer, 
        CompressionType::Best,     // Use best compression for PNG
        FilterType::Adaptive       // Use adaptive filtering for better compression
    );
    
    img.write_with_encoder(encoder)
        .with_context(|| "Failed to encode PNG")?;
    
    Ok(())
}

fn compress_png_aggressive(img: &image::DynamicImage, output_path: &Path) -> Result<()> {
    use image::codecs::png::{PngEncoder, CompressionType, FilterType};
    use std::io::BufWriter;
    
    // Convert to RGB8 to remove alpha channel for smaller file size
    let rgb_img = img.to_rgb8();
    
    let output_file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create PNG file: {}", output_path.display()))?;
    
    let buf_writer = BufWriter::new(output_file);
    let encoder = PngEncoder::new_with_quality(
        buf_writer, 
        CompressionType::Best,
        FilterType::Adaptive
    );
    
    rgb_img.write_with_encoder(encoder)
        .with_context(|| "Failed to encode PNG")?;
    
    Ok(())
}

fn compress_webp(img: &image::DynamicImage, output_path: &Path, quality: u8) -> Result<()> {
    // Convert to RGB8 to strip metadata and ensure compatibility
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    // Use direct encoding for maximum speed
    let webp_data = if quality >= 100 {
        webp::Encoder::from_rgb(&rgb_img, width, height).encode_lossless()
    } else {
        webp::Encoder::from_rgb(&rgb_img, width, height).encode(f32::from(quality))
    };
    
    fs::write(output_path, &*webp_data)
        .with_context(|| format!("Failed to write WebP file: {}", output_path.display()))?;
    
    Ok(())
}

fn print_results(stats: &CompressionStats, processing_time: std::time::Duration, _total_time: std::time::Duration) {
    if !stats.file_results.is_empty() {
        let mut table = Table::new();
        table.set_header(vec!["Filename", "Original", "Compressed", "Savings"]);

        for result in &stats.file_results {
            let original = format_size(result.original_size, DECIMAL);
            let compressed = format_size(result.compressed_size, DECIMAL);
            let savings_bytes = result.original_size.saturating_sub(result.compressed_size);
            let savings = if result.original_size > 0 {
                format!("{} ({:.1}%)", format_size(savings_bytes, DECIMAL), (savings_bytes as f64 / result.original_size as f64) * 100.0)
            } else {
                "0 B (0.0%)".to_string()
            };
            table.add_row(vec![&result.filename, &original, &compressed, &savings]);
        }

        println!("{}", table);
    }

    let original_text = format_size(stats.original_size, DECIMAL);
    let compressed_text = format_size(stats.compressed_size, DECIMAL);
    let savings = stats.savings_percent();
    let savings_bytes = stats.original_size.saturating_sub(stats.compressed_size);
    let savings_text = format_size(savings_bytes, DECIMAL);
    
    println!();
    println!("{} files processed", stats.files_processed.to_string().bright_white().bold());
    println!("Original: {} → Compressed: {}", original_text.bright_cyan(), compressed_text.bright_cyan());
    
    if savings > 0.0 {
        println!("Saved {} ({:.1}%)", savings_text.bright_green(), savings);
    }
    
    println!("Time: {}", format!("{:.2?}", processing_time).bright_cyan());
    
    if !stats.errors.is_empty() {
        println!("{} errors", stats.errors.len());
    }
}


