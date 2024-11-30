use std::path::Path;
use std::fs;
use chardet::{detect, charset2encoding};
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct FileEncoding {
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bom: Option<&'static str>,
}

pub fn detect_bom(content: &[u8]) -> Option<&'static str> {
    if content.len() < 2 {
        return None;
    }
    
    match content {
        [0xEF, 0xBB, 0xBF, ..] => Some("UTF-8"),
        [0xFE, 0xFF, ..] => Some("UTF-16BE"),
        [0xFF, 0xFE, 0x00, 0x00, ..] => Some("UTF-32LE"),
        [0x00, 0x00, 0xFE, 0xFF, ..] => Some("UTF-32BE"),
        [0xFF, 0xFE, ..] => Some("UTF-16LE"),
        _ => None
    }
}

pub fn detect_encoding(path: &Path) -> FileEncoding {
    match fs::read(path) {
        Ok(content) => {
            if content.is_empty() {
                return FileEncoding {
                    encoding: "empty file".to_string(),
                    bom: None,
                };
            }
            let bom = detect_bom(&content);
            let detect_result = detect(&content);
            FileEncoding {
                encoding: charset2encoding(&detect_result.0).to_string(),
                bom,
            }
        }
        Err(_) => FileEncoding {
            encoding: "binary/unreadable".to_string(),
            bom: None,
        }
    }
}
