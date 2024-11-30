mod detection;
mod statistics;
mod filter;

use clap::Parser;
use std::fs;
use std::path::Path;
use std::error::Error;
use detection::detect_encoding;
use statistics::Statistics;
use filter::FileFilter;

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
}

fn scan_directory(
    path: &Path,
    indent: usize,
    stats: &mut Statistics,
    filter: &FileFilter,
) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        println!("{}📁 {}", " ".repeat(indent), name);
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            scan_directory(&path, indent + 2, stats, filter)?;
        }
    } else if filter.should_include(path) {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let encoding = detect_encoding(path);
        let bom_info = encoding.bom.map_or("No BOM".to_string(), |b| format!("BOM: {}", b));
        println!("{}📄 {} [{}, {}]", " ".repeat(indent), name, encoding.encoding, bom_info);
        stats.add_file(encoding);
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    let path = Path::new(&args.path);
    
    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    let filter = FileFilter::new(args.extensions);
    let mut stats = Statistics::new();

    match scan_directory(path, 0, &mut stats, &filter) {
        Ok(_) => {
            println!("\nDirectory scan completed successfully.");
            stats.display_summary();
        }
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    }
}
