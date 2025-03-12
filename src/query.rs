// this_file: fontgrep/src/query.rs
//
// Query execution and font matching

use crate::{
    cli::{parse_codepoints, SearchArgs},
    font::{is_font_file, FontInfo},
    matchers::{
        AxesMatcher, CodepointsMatcher, FeaturesMatcher, FontMatcher, NameMatcher, ScriptsMatcher,
        TablesMatcher,
    },
    Result,
};
use jwalk::WalkDir;
use skrifa::Tag;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Criteria for querying fonts
#[derive(Default)]
pub struct FontQuery {
    matchers: Vec<Box<dyn FontMatcher>>,
    jobs: usize,
    paths: Vec<PathBuf>,
}

// It's fine.
unsafe impl Sync for FontQuery {}

impl From<&SearchArgs> for FontQuery {
    fn from(args: &SearchArgs) -> Self {
        let mut matchers: Vec<Box<dyn FontMatcher>> = Vec::new();

        // Matches should be added from quickest / most effective filter to slowest
        if args.variable {
            matchers.push(Box::new(TablesMatcher::new(&[Tag::new(b"fvar")])));
        }

        if !args.tables.is_empty() {
            matchers.push(Box::new(TablesMatcher::new(&args.tables)));
        }

        if !args.axes.is_empty() {
            matchers.push(Box::new(AxesMatcher::new(&args.axes)));
        }

        if !args.features.is_empty() {
            matchers.push(Box::new(FeaturesMatcher::new(&args.features)));
        }

        if !args.scripts.is_empty() {
            matchers.push(Box::new(ScriptsMatcher::new(&args.scripts)));
        }

        if !args.name.is_empty() {
            matchers.push(Box::new(NameMatcher::new(&args.name)));
        }

        if !args.codepoints.is_empty() || args.text.is_some() {
            let mut codepoints: Vec<char> = Vec::new();
            if let Some(text) = &args.text {
                codepoints.extend(text.chars());
            }

            // Parse the codepoints from strings
            if !args.codepoints.is_empty() {
                let codepoint_str = args.codepoints.join(",");
                if let Ok(parsed_codepoints) = parse_codepoints(&codepoint_str) {
                    codepoints.extend(parsed_codepoints);
                }
            }

            matchers.push(Box::new(CodepointsMatcher::new(&codepoints)));
        }

        Self {
            matchers,
            jobs: args.jobs,
            paths: args.paths.clone(),
        }
    }
}

impl FontQuery {
    /// Execute the query
    pub fn execute(&self, json_output: bool) -> Result<Vec<String>> {
        // For collecting results (needed for JSON output)
        let matching_fonts = Arc::new(Mutex::new(Vec::new()));

        // Process each path
        for path in &self.paths {
            if path.is_file() {
                // If it's a file, process it directly
                if is_font_file(path) {
                    match self.process_font_file(path) {
                        Ok(true) => {
                            // Font matches criteria
                            let path_str = path.to_string_lossy().to_string();
                            if !json_output {
                                println!("{}", path_str);
                            }
                            let mut fonts = matching_fonts.lock().unwrap();
                            fonts.push(path_str);
                        }
                        Ok(false) => {
                            // Font doesn't match criteria
                        }
                        Err(e) => {
                            eprintln!("Error processing font {}: {}", path.display(), e);
                        }
                    }
                }
            } else if path.is_dir() {
                // If it's a directory, walk it recursively using jwalk
                // jwalk is already parallelized internally
                let walker = WalkDir::new(path)
                    .parallelism(jwalk::Parallelism::RayonNewPool(self.jobs))
                    .process_read_dir(move |_depth, _path, _read_dir_state, children| {
                        children.retain(|dir_entry_result| {
                            dir_entry_result
                                .as_ref()
                                .map(|dir_entry| {
                                    dir_entry.file_type().is_dir()
                                        || (dir_entry.file_type().is_file()
                                            && is_font_file(&dir_entry.path()))
                                })
                                .unwrap_or(false)
                        });
                    })
                    .sort(true);

                for entry in walker.into_iter().flatten() {
                    if entry.file_type().is_dir() {
                        continue;
                    }

                    match self.process_font_file(&entry.path()) {
                        Ok(true) => {
                            // Font matches criteria
                            let path_str = entry.path().to_string_lossy().to_string();
                            if !json_output {
                                println!("{}", path_str);
                            }
                            let mut fonts = matching_fonts.lock().unwrap();
                            fonts.push(path_str);
                        }
                        Ok(false) => {
                            // Font doesn't match criteria
                        }
                        Err(e) => {
                            eprintln!("Error processing font {}: {}", entry.path().display(), e);
                        }
                    }
                }
            } else {
                eprintln!("Warning: Path does not exist: {}", path.display());
            }
        }

        // Return the matching fonts
        let result = matching_fonts.lock().unwrap().clone();
        Ok(result)
    }

    /// Process a font file
    fn process_font_file(&self, path: &Path) -> Result<bool> {
        // Load font info
        let font_info = FontInfo::load(path)?;

        Ok(self
            .matchers
            .iter()
            .all(|matcher| matcher.matches(&font_info)))
    }
}
