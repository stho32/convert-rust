use std::collections::HashMap;
use crate::detection::FileEncoding;

pub struct Statistics {
    total_files: usize,
    encoding_counts: HashMap<FileEncoding, usize>,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            total_files: 0,
            encoding_counts: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, encoding: FileEncoding) {
        self.total_files += 1;
        *self.encoding_counts.entry(encoding).or_insert(0) += 1;
    }

    pub fn display_summary(&self) {
        println!("\n=== Encoding Statistics ===");
        println!("Total files scanned: {}", self.total_files);
        println!("\nEncoding Distribution:");
        
        // Sort by count for consistent output
        let mut entries: Vec<_> = self.encoding_counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));

        for (encoding, count) in entries {
            let percentage = (*count as f64 / self.total_files as f64) * 100.0;
            let bom_info = encoding.bom.map_or("No BOM".to_string(), |b| format!("BOM: {}", b));
            println!("- {} ({}) : {} files ({:.1}%)", 
                encoding.encoding, 
                bom_info,
                count, 
                percentage
            );
        }
    }
}
