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

#[derive(Debug)]
pub struct BomInfo {
    bom_type: &'static str,
    skip_bytes: usize,
}

pub fn detect_bom(content: &[u8]) -> Option<BomInfo> {
    if content.len() < 2 {
        return None;
    }
    
    match content {
        [0xEF, 0xBB, 0xBF, ..] => Some(BomInfo { bom_type: "UTF-8", skip_bytes: 3 }),
        [0xFE, 0xFF, ..] => Some(BomInfo { bom_type: "UTF-16BE", skip_bytes: 2 }),
        [0xFF, 0xFE, 0x00, 0x00, ..] => Some(BomInfo { bom_type: "UTF-32LE", skip_bytes: 4 }),
        [0x00, 0x00, 0xFE, 0xFF, ..] => Some(BomInfo { bom_type: "UTF-32BE", skip_bytes: 4 }),
        [0xFF, 0xFE, ..] => Some(BomInfo { bom_type: "UTF-16LE", skip_bytes: 2 }),
        _ => None
    }
}

fn is_ascii(content: &[u8]) -> bool {
    content.iter().all(|&b| b < 128)
}

fn is_utf8(content: &[u8]) -> bool {
    String::from_utf8(content.to_vec()).is_ok()
}

fn looks_like_windows1252_or_iso8859_1(content: &[u8]) -> bool {
    // Check if the content contains bytes in the Windows-1252 specific range
    content.iter().any(|&b| matches!(b, 0x80..=0x9F)) ||
    // Check for common Windows-1252/ISO-8859-1 characters
    content.iter().any(|&b| matches!(b, 0xA0..=0xFF))
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

            // First check for BOM
            if let Some(bom_info) = detect_bom(&content) {
                return FileEncoding {
                    encoding: bom_info.bom_type.to_string(),
                    bom: Some(bom_info.bom_type),
                };
            }

            // If no BOM, try to detect encoding
            let content_without_bom = if let Some(bom_info) = detect_bom(&content) {
                &content[bom_info.skip_bytes..]
            } else {
                &content
            };

            // Check for ASCII first (subset of UTF-8)
            if is_ascii(content_without_bom) {
                return FileEncoding {
                    encoding: "ASCII".to_string(),
                    bom: None,
                };
            }

            // Check for UTF-8 without BOM
            if is_utf8(content_without_bom) {
                return FileEncoding {
                    encoding: "UTF-8".to_string(),
                    bom: None,
                };
            }

            // Use chardet for additional detection
            let detect_result = detect(content_without_bom);
            let chardet_encoding = charset2encoding(&detect_result.0).to_string();

            // Special handling for Windows-1252 and ISO-8859-1
            if chardet_encoding == "ISO-8859-1" || chardet_encoding == "windows-1252" {
                if looks_like_windows1252_or_iso8859_1(content_without_bom) {
                    let encoding = if content_without_bom.iter().any(|&b| matches!(b, 0x80..=0x9F)) {
                        "windows-1252"
                    } else {
                        "ISO-8859-1"
                    };
                    return FileEncoding {
                        encoding: encoding.to_string(),
                        bom: None,
                    };
                }
            }

            // Return chardet result if no other encoding was detected
            FileEncoding {
                encoding: chardet_encoding,
                bom: None,
            }
        }
        Err(_) => FileEncoding {
            encoding: "binary/unreadable".to_string(),
            bom: None,
        }
    }
}
