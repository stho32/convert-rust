mod detection;
mod statistics;
mod filter;
mod output;

use clap::Parser;
use std::fs;
use std::path::Path;
use std::error::Error;
use detection::detect_encoding;
use statistics::Statistics;
use filter::FileFilter;
use output::OutputFormat;

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
}

fn scan_directory(
    path: &Path,
    indent: usize,
    stats: &mut Statistics,
    filter: &FileFilter,
    format: &OutputFormat,
) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        if matches!(format, OutputFormat::Text) {
            println!("{}📁 {}", " ".repeat(indent), name);
        }
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            scan_directory(&path, indent + 2, stats, filter, format)?;
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

    let filter = FileFilter::new(args.extensions);
    let mut stats = Statistics::new();

    match scan_directory(path, 0, &mut stats, &filter, &format) {
        Ok(_) => {
            if matches!(format, OutputFormat::Text) {
                println!("\nDirectory scan completed successfully.");
            }
            stats.display_summary(&format)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    }
}
