use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read};
use chrono::Local;
use log::{info, warn};
use crate::detection::{detect_encoding, FileEncoding};

#[derive(Debug)]
pub enum SafetyError {
    IoError(io::Error),
    VerificationFailed(String),
    BackupFailed(String),
    RollbackFailed(String),
}

impl std::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyError::IoError(e) => write!(f, "IO error: {}", e),
            SafetyError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
            SafetyError::BackupFailed(msg) => write!(f, "Backup failed: {}", msg),
            SafetyError::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
        }
    }
}

impl std::error::Error for SafetyError {}

impl From<io::Error> for SafetyError {
    fn from(error: io::Error) -> Self {
        SafetyError::IoError(error)
    }
}

pub struct ConversionSafety {
    backup_dir: Option<PathBuf>,
    log_file: PathBuf,
    input_dir: PathBuf,
    create_backup: bool,
}

impl ConversionSafety {
    pub fn new(input_dir: &Path, output_dir: &Path, create_backup: bool) -> Result<Self, SafetyError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_dir = if create_backup {
            let dir = output_dir.join(format!("backup_{}", timestamp));
            fs::create_dir_all(&dir)?;
            Some(dir)
        } else {
            None
        };
        
        let log_file = output_dir.join(format!("conversion_log_{}.txt", timestamp));
        
        let safety = ConversionSafety {
            backup_dir,
            log_file,
            input_dir: input_dir.to_path_buf(),
            create_backup,
        };
        
        safety.init_logging()?;
        Ok(safety)
    }

    fn init_logging(&self) -> Result<(), SafetyError> {
        // Create parent directory for log file if it doesn't exist
        if let Some(parent) = self.log_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let log_file = File::create(&self.log_file)?;
        
        let config = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}] {}",
                    Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.level(),
                    message
                ))
            })
            .chain(log_file)
            .chain(io::stdout())
            .level(log::LevelFilter::Info)
            .level_for("convert_rust", log::LevelFilter::Debug);

        config.apply().map_err(|e| SafetyError::IoError(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to initialize logging: {}", e)
        )))?;

        info!("Conversion process started");
        Ok(())
    }

    pub fn create_backup(&self, file_path: &Path) -> Result<Option<PathBuf>, SafetyError> {
        if !self.create_backup {
            return Ok(None);
        }

        // Get the relative path from input_dir to file_path
        let rel_path = file_path.strip_prefix(&self.input_dir)
            .map_err(|_| SafetyError::BackupFailed(
                format!("File {} is not within input directory {}", 
                    file_path.display(), self.input_dir.display())
            ))?;

        // Create the backup path with preserved directory structure
        let backup_path = self.backup_dir.as_ref()
            .ok_or_else(|| SafetyError::BackupFailed("Backup directory not initialized".to_string()))?
            .join(rel_path);

        // Create parent directories if they don't exist
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(file_path, &backup_path)?;
        info!("Created backup of {} at {}", file_path.display(), backup_path.display());
        Ok(Some(backup_path))
    }

    pub fn verify_conversion(&self, original: &Path, converted: &Path) -> Result<(), SafetyError> {
        info!("Verifying conversion of {}", original.display());

        // Check if the converted file exists and is readable
        let mut converted_content = Vec::new();
        File::open(converted)?.read_to_end(&mut converted_content)?;

        if converted_content.is_empty() {
            return Err(SafetyError::VerificationFailed(
                "Converted file is empty".to_string()
            ));
        }

        // Detect encoding of converted file
        let encoding = detect_encoding(converted);
        
        // Verify the converted file is readable with its encoding
        self.verify_file_readability(converted, &encoding)?;

        info!("Conversion verification successful for {}", original.display());
        Ok(())
    }

    fn verify_file_readability(&self, path: &Path, encoding: &FileEncoding) -> Result<(), SafetyError> {
        let content = fs::read(path)?;
        
        // Try to decode the content using the detected encoding
        let decoder = encoding_rs::Encoding::for_label(encoding.encoding.as_bytes())
            .ok_or_else(|| SafetyError::VerificationFailed(
                format!("Invalid encoding: {}", encoding.encoding)
            ))?;

        let (cow, had_errors) = decoder.decode_without_bom_handling(&content);
        
        if had_errors {
            return Err(SafetyError::VerificationFailed(
                format!("File {} is not readable with encoding {}", 
                    path.display(), encoding.encoding)
            ));
        }

        // Check if the decoded content is valid UTF-8
        if !cow.chars().all(|c| !c.is_control() || c.is_whitespace()) {
            return Err(SafetyError::VerificationFailed(
                format!("File {} contains invalid characters", path.display())
            ));
        }

        Ok(())
    }

    pub fn rollback(&self, original: &Path, backup: &Path) -> Result<(), SafetyError> {
        if !self.create_backup {
            return Ok(());
        }

        warn!("Rolling back changes for {}", original.display());
        
        // Create parent directories if they don't exist
        if let Some(parent) = original.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy backup back to original location
        fs::copy(backup, original).map_err(|e| SafetyError::RollbackFailed(
            format!("Failed to restore backup: {}", e)
        ))?;

        info!("Successfully rolled back changes for {}", original.display());
        Ok(())
    }

    pub fn get_backup_dir(&self) -> Option<&Path> {
        self.backup_dir.as_deref()
    }

    pub fn get_log_file(&self) -> &Path {
        &self.log_file
    }
}
