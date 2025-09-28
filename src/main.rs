use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::*;
use humansize::{format_size, DECIMAL};
use image::GenericImageView;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
        default_value = "65",
        help = "Compression quality (1-100)"
    )]
    quality: u8,

    #[arg(
        long,
        default_value = "5",
        help = "Minimum compression savings percentage to keep file (0-100)"
    )]
    min_savings: f64,

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

#[derive(Debug, Clone)]
struct FileResult {
    filename: String,
    original_size: u64,
    compressed_size: u64,
    savings_percent: f64,
    status: String,
    skipped: bool,
}

#[derive(Debug)]
struct CompressionStats {
    files_processed: usize,
    original_size: u64,
    compressed_size: u64,
    errors: Vec<String>,
    file_results: Vec<FileResult>,
}

impl CompressionStats {
    fn new() -> Self {
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
        println!();
        println!("{}", "No image files found".bright_red().bold());
        println!();
        println!("{} JPEG, PNG, WebP, BMP, TIFF, GIF", "Supported formats:".bright_yellow());
        println!();
        println!("{}:", "Suggestions".bright_yellow());
        println!("  - Check if the path is correct");
        println!("  - Use --recursive flag for subdirectories");
        println!();
        return Ok(());
    }

    println!("Found {} image{} to process", 
        files.len().to_string().bright_green().bold(),
        if files.len() == 1 { "" } else { "s" }
    );
    println!();

    let pb = create_progress_bar(files.len());
    let stats = Arc::new(Mutex::new(CompressionStats::new()));
    let pb_arc = Arc::new(pb);

    // Process files in parallel for better performance
    files.par_iter().for_each(|file_path| {
        let filename = file_path.file_name().unwrap().to_string_lossy().to_string();
        pb_arc.set_message(format!("{}", filename));

        match compress_image(file_path, &output_dir, &args) {
            Ok((original_size, compressed_size, skipped)) => {
                let savings = if original_size > 0 {
                    let saved = original_size.saturating_sub(compressed_size) as f64;
                    (saved / original_size as f64) * 100.0
                } else {
                    0.0
                };

                let status = if skipped {
                    "Skipped".to_string()
                } else if savings > 0.0 {
                    "Compressed".to_string()
                } else if savings < 0.0 {
                    "Enlarged".to_string()
                } else {
                    "No change".to_string()
                };

                let file_result = FileResult {
                    filename,
                    original_size,
                    compressed_size,
                    savings_percent: savings,
                    status,
                    skipped,
                };

                let mut stats_guard = stats.lock().unwrap();
                stats_guard.add_file_result(file_result);
            }
            Err(e) => {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.errors.push(format!("{}: {}", filename, e));
            }
        }

        pb_arc.inc(1);
    });

    pb_arc.finish_with_message("Compression complete");
    let final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();
    print_results(&final_stats);

    Ok(())
}

fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    let author = "SujalXplores";
    
    println!("------------------------------------------");
    println!("{} {}", "PixelSqueeze".bright_white().bold(), format!("v{}", version).bright_green());
    println!("{}", "High-performance image compression".bright_cyan());
    println!("Created by {}", author.bright_magenta());
    println!("------------------------------------------");
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
            .template("[{elapsed_precise}] {bar:50} {pos:>3}/{len:3} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb
}

fn estimate_jpeg_needs_compression(input_path: &Path, target_quality: u8) -> Result<bool> {
    // Simple heuristic: check file size vs dimensions ratio
    let metadata = fs::metadata(input_path)?;
    let file_size = metadata.len();
    
    // Open image to get dimensions
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();
    let pixels = (width * height) as u64;
    
    // Calculate bytes per pixel - lower values suggest higher compression
    let bytes_per_pixel = file_size as f64 / pixels as f64;
    
    // Rough quality estimation based on bytes per pixel
    // High quality JPEGs: > 1.5 bytes/pixel
    // Medium quality: 0.5-1.5 bytes/pixel  
    // Low quality: < 0.5 bytes/pixel
    let estimated_quality = if bytes_per_pixel > 1.5 {
        85
    } else if bytes_per_pixel > 1.0 {
        75
    } else if bytes_per_pixel > 0.5 {
        65
    } else {
        50
    };
    
    // Only compress if target quality is significantly lower than estimated
    Ok(target_quality < estimated_quality - 10)
}

fn compress_image(input_path: &Path, output_dir: &Path, args: &Args) -> Result<(u64, u64, bool)> {
    let original_size = fs::metadata(input_path)?.len();

    // Check if it's already a JPEG and estimate its quality
    let should_compress_jpeg = if let Some(ext) = input_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if matches!(ext_str.as_str(), "jpg" | "jpeg") {
            estimate_jpeg_needs_compression(input_path, args.quality)?
        } else {
            true
        }
    } else {
        true
    };

    if !should_compress_jpeg && matches!(args.format, OutputFormat::Jpeg) {
        // Skip compression for already well-compressed JPEGs
        return Ok((original_size, original_size, true));
    }

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
            let writer = BufWriter::with_capacity(64 * 1024, output_file);
            
            // Create JPEG encoder with quality setting
            let encoder = JpegEncoder::new_with_quality(writer, args.quality);
            img.write_with_encoder(encoder)?;
        }
        OutputFormat::Png => {
            use image::codecs::png::{PngEncoder, CompressionType};
            use std::io::BufWriter;
            
            let output_file = fs::File::create(&output_path)?;
            let writer = BufWriter::with_capacity(64 * 1024, output_file);
            
            // Use maximum PNG compression
            let encoder = PngEncoder::new_with_quality(writer, CompressionType::Best, image::codecs::png::FilterType::Adaptive);
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
    
    // Check if compression meets minimum savings threshold
    let savings_percent = if original_size > 0 {
        let saved = original_size.saturating_sub(compressed_size) as f64;
        (saved / original_size as f64) * 100.0
    } else {
        0.0
    };
    
    // If compression didn't meet threshold, remove the compressed file and return original
    if savings_percent < args.min_savings && compressed_size >= original_size {
        fs::remove_file(&output_path).ok(); // Ignore errors when cleaning up
        return Ok((original_size, original_size, true));
    }
    
    Ok((original_size, compressed_size, false))
}

fn print_results(stats: &CompressionStats) {
    println!();
    
    // Individual file results in table format
    if !stats.file_results.is_empty() {
        println!("{}", "Individual File Results:".bright_white().bold());
        println!();

        // Table header
        println!("{:<4} {:<30} {:<12} {:<12} {:<12} {:<15}", 
            "#".bright_blue().bold(),
            "Filename".bright_blue().bold(),
            "Original".bright_blue().bold(),
            "Compressed".bright_blue().bold(),
            "Saved".bright_blue().bold(),
            "Status".bright_blue().bold()
        );
        println!("{}", "-".repeat(95).bright_black());

        for (i, result) in stats.file_results.iter().enumerate() {
            let original_size_str = format_size(result.original_size, DECIMAL);
            let compressed_size_str = format_size(result.compressed_size, DECIMAL);
            
            // Calculate savings
            let savings_bytes = if result.original_size > result.compressed_size {
                result.original_size - result.compressed_size
            } else {
                0
            };
            let savings_str = format_size(savings_bytes, DECIMAL);

            // Truncate filename if too long
            let display_filename = if result.filename.len() > 28 {
                format!("{}...", &result.filename[..25])
            } else {
                result.filename.clone()
            };

            let status_colored = if result.skipped {
                result.status.bright_blue()
            } else if result.savings_percent > 0.0 {
                format!("{} ({:.1}%)", result.status, result.savings_percent).bright_green()
            } else if result.savings_percent < 0.0 {
                format!("{} ({:.1}%)", result.status, -result.savings_percent).bright_red()
            } else {
                result.status.bright_yellow()
            };

            println!("{:<4} {:<30} {:<12} {:<12} {:<12} {}", 
                format!("{}.", i + 1).bright_white(),
                display_filename.bright_white(),
                original_size_str.bright_cyan(),
                compressed_size_str.bright_cyan(),
                if savings_bytes > 0 { savings_str.bright_green() } else { "-".bright_black() },
                status_colored
            );
        }
        
        println!();
        println!("{}", "-".repeat(95).bright_black());
    }

    // Overall summary
    println!();
    println!("{}", "Summary:".bright_white().bold());
    println!();

    let original_text = format_size(stats.original_size, DECIMAL);
    let compressed_text = format_size(stats.compressed_size, DECIMAL);

    // Count skipped files
    let skipped_count = stats.file_results.iter().filter(|r| r.skipped).count();
    let compressed_count = stats.files_processed - skipped_count;

    // Summary table
    println!("{:<25} {}", "Files processed:".bright_yellow(), stats.files_processed.to_string().bright_white().bold());
    println!("{:<25} {}", "Files compressed:".bright_yellow(), compressed_count.to_string().bright_green());
    println!("{:<25} {}", "Files skipped:".bright_yellow(), skipped_count.to_string().bright_blue());
    println!("{:<25} {}", "Total original size:".bright_yellow(), original_text.bright_cyan());
    println!("{:<25} {}", "Total compressed size:".bright_yellow(), compressed_text.bright_cyan());

    let savings = stats.savings_percent();
    let savings_bytes = if stats.original_size > stats.compressed_size {
        stats.original_size - stats.compressed_size
    } else {
        0
    };
    let savings_text = format_size(savings_bytes, DECIMAL);
    
    let change_text = if savings > 0.0 {
        format!("{:.1}% smaller (saved {})", savings, savings_text).bright_green()
    } else if savings < 0.0 {
        format!("{:.1}% larger", -savings).bright_red()
    } else {
        "No change".bright_yellow()
    };

    println!("{:<25} {}", "Space change:".bright_yellow(), change_text);

    // Add compression ratio
    let ratio = if stats.original_size > 0 {
        stats.compressed_size as f64 / stats.original_size as f64
    } else {
        1.0
    };
    println!("{:<25} {}", "Compression ratio:".bright_yellow(), format!("{:.2}:1", 1.0/ratio).bright_magenta());

    // Show errors if any
    if !stats.errors.is_empty() {
        println!();
        println!("{} {} error{} encountered:", 
            "Warning:".bright_red().bold(),
            stats.errors.len(),
            if stats.errors.len() == 1 { "" } else { "s" }
        );
        for error in &stats.errors {
            println!("  - {}", error.bright_red());
        }
    }

    // Success message
    if stats.files_processed > 0 {
        println!();
        let success_msg = if savings > 0.0 {
            format!("Successfully compressed {} file{} and saved {} of storage space", 
                stats.files_processed,
                if stats.files_processed == 1 { "" } else { "s" },
                savings_text
            )
        } else {
            format!("Successfully processed {} file{}", 
                stats.files_processed,
                if stats.files_processed == 1 { "" } else { "s" }
            )
        };
        
        println!("{}", success_msg.bright_green().bold());
    }
    println!();
}


