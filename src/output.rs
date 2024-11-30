use std::path::PathBuf;
use serde::Serialize;
use crate::detection::FileEncoding;

#[derive(Serialize, Clone)]
pub struct FileReport {
    pub path: PathBuf,
    pub name: String,
    #[serde(flatten)]
    pub encoding: FileEncoding,
}

#[derive(Serialize)]
pub struct ScanReport {
    pub total_files: usize,
    pub files: Vec<FileReport>,
    pub encoding_stats: Vec<EncodingStat>,
}

#[derive(Serialize, Clone)]
pub struct EncodingStat {
    pub encoding: String,
    pub bom: Option<&'static str>,
    pub count: usize,
    pub percentage: f64,
}

// CSV-specific record types
#[derive(Serialize)]
struct FileRecordCsv {
    path: String,
    name: String,
    encoding: String,
    bom: String,
}

#[derive(Serialize)]
struct StatRecordCsv {
    encoding: String,
    bom: String,
    count: usize,
    percentage: f64,
}

pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" => Some(OutputFormat::Text),
            "json" => Some(OutputFormat::Json),
            "csv" => Some(OutputFormat::Csv),
            _ => None,
        }
    }
}

pub fn write_output(report: &ScanReport, format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Text => write_text_output(report),
        OutputFormat::Json => write_json_output(report),
        OutputFormat::Csv => write_csv_output(report),
    }
}

fn write_text_output(report: &ScanReport) -> Result<(), Box<dyn std::error::Error>> {
    println!("Files:");
    for file in &report.files {
        let bom_info = file.encoding.bom.map_or("No BOM".to_string(), |b| format!("BOM: {}", b));
        println!("📄 {} [{}, {}]", 
            file.path.display(), 
            file.encoding.encoding, 
            bom_info
        );
    }

    println!("\n=== Encoding Statistics ===");
    println!("Total files scanned: {}", report.total_files);
    println!("\nEncoding Distribution:");
    
    for stat in &report.encoding_stats {
        let bom_info = stat.bom.map_or("No BOM".to_string(), |b| format!("BOM: {}", b));
        println!("- {} ({}) : {} files ({:.1}%)", 
            stat.encoding,
            bom_info,
            stat.count,
            stat.percentage
        );
    }
    Ok(())
}

fn write_json_output(report: &ScanReport) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

fn write_csv_output(report: &ScanReport) -> Result<(), Box<dyn std::error::Error>> {
    println!("File Analysis:");
    let mut writer = csv::Writer::from_writer(std::io::stdout());
    
    // Write header manually for clarity
    writer.write_record(&["Path", "Name", "Encoding", "BOM"])?;
    
    // Write files data
    for file in &report.files {
        let record = FileRecordCsv {
            path: file.path.to_string_lossy().to_string(),
            name: file.name.clone(),
            encoding: file.encoding.encoding.clone(),
            bom: file.encoding.bom.map_or("No BOM".to_string(), |b| b.to_string()),
        };
        writer.serialize(record)?;
    }
    writer.flush()?;
    
    // Write encoding statistics
    println!("\nEncoding Statistics:");
    let mut stats_writer = csv::Writer::from_writer(std::io::stdout());
    
    // Write header for stats
    stats_writer.write_record(&["Encoding", "BOM", "Count", "Percentage"])?;
    
    for stat in &report.encoding_stats {
        let record = StatRecordCsv {
            encoding: stat.encoding.clone(),
            bom: stat.bom.map_or("No BOM".to_string(), |b| b.to_string()),
            count: stat.count,
            percentage: stat.percentage,
        };
        stats_writer.serialize(record)?;
    }
    stats_writer.flush()?;
    Ok(())
}
