use std::collections::HashMap;
use std::path::PathBuf;
use crate::detection::FileEncoding;
use crate::output::{FileReport, ScanReport, EncodingStat, OutputFormat, write_output};

pub struct Statistics {
    total_files: usize,
    encoding_counts: HashMap<FileEncoding, usize>,
    files: Vec<FileReport>,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            total_files: 0,
            encoding_counts: HashMap::new(),
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, name: String, encoding: FileEncoding) {
        self.total_files += 1;
        *self.encoding_counts.entry(encoding.clone()).or_insert(0) += 1;
        
        self.files.push(FileReport {
            path,
            name,
            encoding,
        });
    }

    pub fn generate_report(&self) -> ScanReport {
        let mut stats = Vec::new();
        let mut entries: Vec<_> = self.encoding_counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));

        for (encoding, count) in entries {
            let percentage = (*count as f64 / self.total_files as f64) * 100.0;
            stats.push(EncodingStat {
                encoding: encoding.encoding.clone(),
                bom: encoding.bom,
                count: *count,
                percentage,
            });
        }

        ScanReport {
            total_files: self.total_files,
            files: self.files.clone(),
            encoding_stats: stats,
        }
    }

    pub fn display_summary(&self, format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.generate_report();
        write_output(&report, format)
    }
}
