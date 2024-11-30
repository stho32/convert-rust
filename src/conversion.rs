use std::io::{self, Read, Write};
use std::fmt;
use encoding_rs::*;
use crate::detection::FileEncoding;

#[derive(Debug)]
pub enum ConversionError {
    IoError(io::Error),
    EncodingError(String),
    UnsupportedEncoding(String),
}

impl std::error::Error for ConversionError {}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::IoError(e) => write!(f, "IO error: {}", e),
            ConversionError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            ConversionError::UnsupportedEncoding(enc) => write!(f, "Unsupported encoding: {}", enc),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineEnding {
    Unix,    // \n
    Windows, // \r\n
    Keep,    // Keep original
}

impl LineEnding {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unix" | "lf" => Some(LineEnding::Unix),
            "windows" | "crlf" => Some(LineEnding::Windows),
            "keep" => Some(LineEnding::Keep),
            _ => None,
        }
    }
}

pub struct EncodingConverter;

impl EncodingConverter {
    pub fn convert(
        input: &[u8], 
        from: &FileEncoding, 
        to: &str,
        line_ending: LineEnding,
    ) -> Result<Vec<u8>, ConversionError> {
        let (decoder, encoder) = Self::get_codecs(from, to)?;
        
        // Decode from source encoding to UTF-8
        let (cow, _, had_errors) = decoder.decode(input);
        if had_errors {
            return Err(ConversionError::EncodingError(
                format!("Failed to decode from {}", from.encoding)
            ));
        }

        // Convert line endings if needed
        let content = match line_ending {
            LineEnding::Keep => cow.into_owned(),
            LineEnding::Unix => Self::convert_to_unix_endings(&cow),
            LineEnding::Windows => Self::convert_to_windows_endings(&cow),
        };

        // Encode to target encoding
        let (output, _, had_errors) = encoder.encode(&content);
        if had_errors {
            return Err(ConversionError::EncodingError(
                format!("Failed to encode to {}", to)
            ));
        }

        // Add BOM if needed
        let mut result = Vec::new();
        if to == "UTF-8-BOM" || to == "UTF-16LE" || to == "UTF-16BE" {
            result.extend(Self::get_bom(to));
        }
        result.extend(output.into_owned());
        
        Ok(result)
    }

    fn convert_to_unix_endings(text: &str) -> String {
        // First convert all Windows line endings (\r\n) to Unix (\n)
        let text = text.replace("\r\n", "\n");
        // Then convert any remaining Mac line endings (\r) to Unix (\n)
        text.replace('\r', "\n")
    }

    fn convert_to_windows_endings(text: &str) -> String {
        // First convert all line endings to Unix style
        let unix_text = Self::convert_to_unix_endings(text);
        // Then convert all Unix line endings to Windows style
        unix_text.replace('\n', "\r\n")
    }

    fn get_codecs(from: &FileEncoding, to: &str) -> Result<(&'static Encoding, &'static Encoding), ConversionError> {
        let source_enc = Self::get_encoding(&from.encoding)?;
        let target_enc = Self::get_encoding(to)?;
        Ok((source_enc, target_enc))
    }

    fn get_encoding(name: &str) -> Result<&'static Encoding, ConversionError> {
        match name.to_uppercase().as_str() {
            "UTF-8" | "UTF-8-BOM" => Ok(UTF_8),
            "UTF-16LE" => Ok(UTF_16LE),
            "UTF-16BE" => Ok(UTF_16BE),
            "WINDOWS-1252" => Ok(WINDOWS_1252),
            "ISO-8859-1" => Ok(WINDOWS_1252), // ISO-8859-1 is a subset of Windows-1252
            "ASCII" => Ok(UTF_8), // ASCII is a subset of UTF-8
            _ => Err(ConversionError::UnsupportedEncoding(
                name.to_string()
            )),
        }
    }

    fn get_bom(encoding: &str) -> Vec<u8> {
        match encoding.to_uppercase().as_str() {
            "UTF-8-BOM" => vec![0xEF, 0xBB, 0xBF],
            "UTF-16LE" => vec![0xFF, 0xFE],
            "UTF-16BE" => vec![0xFE, 0xFF],
            _ => vec![],
        }
    }

    pub fn convert_file(
        input_path: &std::path::Path,
        output_path: &std::path::Path,
        from: &FileEncoding,
        to: &str,
        line_ending: LineEnding,
    ) -> Result<(), ConversionError> {
        // Read input file
        let mut input = Vec::new();
        let mut file = std::fs::File::open(input_path)
            .map_err(|e| ConversionError::IoError(e))?;
        file.read_to_end(&mut input)
            .map_err(|e| ConversionError::IoError(e))?;

        // Convert content
        let output = Self::convert(&input, from, to, line_ending)?;

        // Write output file
        let mut file = std::fs::File::create(output_path)
            .map_err(|e| ConversionError::IoError(e))?;
        file.write_all(&output)
            .map_err(|e| ConversionError::IoError(e))?;

        Ok(())
    }
}
