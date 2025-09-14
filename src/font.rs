// this_file: fontgrep/src/font.rs
//
// Font information extraction and matching

use crate::{FontgrepError, Result};
use memmap2::Mmap;
use skrifa::{raw::FileRef, FontRef};
use std::{fs::File, path::Path};

/// Font information extracted from a font file
pub struct FileInfo {
    pub font_data: Mmap,
    // pub(crate) font: FontRef<'a>,
}

pub struct FontInfo<'a> {
    pub font: FontRef<'a>,
}

impl FileInfo {
    /// Load font information from a file
    pub fn load(path: &Path) -> Result<FileInfo> {
        let file = File::open(path)?;
        let data = unsafe { Mmap::map(&file).map_err(|e| FontgrepError::Mmap(e.to_string()))? };

        // Check we can do the thing.
        FileRef::new(&data).map_err(|e| FontgrepError::Font(e.to_string()))?;

        Ok(Self { font_data: data })
    }

    pub fn fonts(&self) -> impl Iterator<Item = FontInfo<'_>> {
        // We already checked we can do the thing
        FileRef::new(&self.font_data)
            .unwrap()
            .fonts()
            .flatten()
            .map(|font| FontInfo { font })
    }
}

/// Check if a file is a font based on its extension
pub(crate) fn is_font_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "ttf" | "otf" | "ttc" | "otc")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_font_file() {
        assert!(is_font_file(Path::new("test.ttf")));
        assert!(is_font_file(Path::new("test.otf")));
        assert!(is_font_file(Path::new("test.ttc")));
        assert!(is_font_file(Path::new("test.otc")));
        assert!(is_font_file(Path::new("test.TTF")));

        assert!(!is_font_file(Path::new("test.txt")));
        assert!(!is_font_file(Path::new("test")));
    }
}
