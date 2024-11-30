use std::path::Path;

pub struct FileFilter {
    extensions: Option<Vec<String>>,
}

impl FileFilter {
    pub fn new(extensions: Option<Vec<String>>) -> Self {
        let extensions = extensions.map(|exts| {
            exts.into_iter()
                .map(|ext| {
                    let ext = ext.trim_start_matches('.');
                    ext.to_lowercase()
                })
                .collect()
        });
        
        FileFilter { extensions }
    }

    pub fn should_include(&self, path: &Path) -> bool {
        if let Some(extensions) = &self.extensions {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    return extensions.contains(&ext_str.to_lowercase());
                }
            }
            false // No extension or invalid UTF-8 extension
        } else {
            true // No filter means include all files
        }
    }
}
