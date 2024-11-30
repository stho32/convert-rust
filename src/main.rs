mod detection;
mod statistics;
mod filter;
mod output;
mod conversion;

use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use detection::detect_encoding;
use statistics::Statistics;
use filter::FileFilter;
use output::OutputFormat;
use conversion::{EncodingConverter, LineEnding};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to process
    #[arg(short, long)]
    path: String,

    /// File extensions to include (e.g., "txt,md,rs")
    /// If not specified, all files will be included
    #[arg(short, long, value_delimiter = ',')]
    extensions: Option<Vec<String>>,

    /// Output format (text, json, or csv)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Convert files to specified encoding
    /// Supported: UTF-8, UTF-8-BOM, UTF-16LE, UTF-16BE, WINDOWS-1252, ISO-8859-1, ASCII
    #[arg(short = 'c', long)]
    convert_to: Option<String>,

    /// Line ending to use (unix, windows, keep)
    #[arg(short = 'l', long, default_value = "unix")]
    line_ending: String,

    /// Output directory for converted files
    #[arg(short = 'o', long)]
    output_dir: Option<String>,
}

fn scan_directory(
    path: &Path,
    stats: &mut Statistics,
    filter: &FileFilter,
) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            scan_directory(&path, stats, filter)?;
        }
    } else if filter.should_include(path) {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let encoding = detect_encoding(path);
        stats.add_file(path.to_path_buf(), name, encoding);
    }
    Ok(())
}

fn convert_files(
    files: &[(PathBuf, String, detection::FileEncoding)],
    target_encoding: &str,
    line_ending: LineEnding,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output_dir)?;

    for (path, name, encoding) in files {
        let output_path = output_dir.join(name);
        match EncodingConverter::convert_file(path, &output_path, encoding, target_encoding, line_ending) {
            Ok(_) => println!("✓ Converted {} to {} with {} line endings", 
                path.display(), 
                target_encoding,
                match line_ending {
                    LineEnding::Unix => "Unix",
                    LineEnding::Windows => "Windows",
                    LineEnding::Keep => "original",
                }
            ),
            Err(e) => eprintln!("✗ Failed to convert {}: {:?}", path.display(), e),
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    
    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    let format = OutputFormat::from_str(&args.format).ok_or_else(|| {
        format!("Invalid output format: '{}'. Valid formats are: text, json, csv", args.format)
    })?;

    let line_ending = LineEnding::from_str(&args.line_ending).ok_or_else(|| {
        format!("Invalid line ending: '{}'. Valid options are: unix, windows, keep", args.line_ending)
    })?;

    let filter = FileFilter::new(args.extensions);
    let mut stats = Statistics::new();

    match scan_directory(path, &mut stats, &filter) {
        Ok(_) => {
            if matches!(format, OutputFormat::Text) {
                println!("\nDirectory scan completed successfully.");
            }
            stats.display_summary(&format)?;

            // Handle conversion if requested
            if let Some(target_encoding) = args.convert_to {
                let output_dir = args.output_dir.map(PathBuf::from)
                    .unwrap_or_else(|| {
                        let mut dir = path.to_path_buf();
                        dir.push("converted");
                        dir
                    });

                println!("\nConverting files to {} with {} line endings...", 
                    target_encoding,
                    match line_ending {
                        LineEnding::Unix => "Unix",
                        LineEnding::Windows => "Windows",
                        LineEnding::Keep => "original",
                    }
                );
                convert_files(&stats.get_files(), &target_encoding, line_ending, &output_dir)?;
                println!("\nConversion completed. Output directory: {}", output_dir.display());
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    }
}
