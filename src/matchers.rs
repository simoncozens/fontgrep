use crate::font::FontInfo;
use itertools::Either;
use skrifa::{raw::TableProvider, MetadataProvider, Tag};
use std::collections::HashSet;

/// Trait for matching fonts
pub trait FontMatcher {
    /// Check if a font matches the criteria
    fn matches(&self, info: &FontInfo) -> bool;
}

/// Matcher for variation axes
pub(crate) struct AxesMatcher {
    axes: Vec<String>,
}

impl AxesMatcher {
    /// Create a new axes matcher
    pub fn new(axes: &[String]) -> Self {
        Self {
            axes: axes.to_vec(),
        }
    }

    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = String> + 'a {
        info.font.axes().iter().map(|axis| axis.tag().to_string())
    }
}

impl FontMatcher for AxesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let all_axes: HashSet<String> = self.extract(info).collect();
        self.axes.iter().all(|axis| all_axes.contains(axis))
    }
}

/// Matcher for OpenType features
pub(crate) struct FeaturesMatcher {
    wanted_features: Vec<String>,
}

impl FeaturesMatcher {
    /// Create a new features matcher
    pub fn new(wanted_features: &[String]) -> Self {
        Self {
            wanted_features: wanted_features.to_vec(),
        }
    }
    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = String> + 'a {
        // Extract GSUB features
        let gsub_features = info
            .font
            .gsub()
            .and_then(|gsub| gsub.feature_list())
            .map(|feature_list| feature_list.feature_records())
            .into_iter()
            .flatten()
            .map(|feature| feature.feature_tag().to_string());
        // Extract GPOS features
        let gpos_features = info
            .font
            .gpos()
            .and_then(|gpos| gpos.feature_list())
            .map(|feature_list| feature_list.feature_records())
            .into_iter()
            .flatten()
            .map(|feature| feature.feature_tag().to_string());
        gsub_features.chain(gpos_features)
    }
}

impl FontMatcher for FeaturesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let all_features: HashSet<String> = self.extract(info).collect();
        self.wanted_features
            .iter()
            .all(|feature| all_features.contains(feature))
    }
}

/// Matcher for OpenType scripts
pub(crate) struct ScriptsMatcher {
    wanted_scripts: Vec<String>,
}

impl ScriptsMatcher {
    /// Create a new scripts matcher
    pub fn new(scripts: &[String]) -> Self {
        Self {
            wanted_scripts: scripts.to_vec(),
        }
    }

    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = String> + 'a {
        // Extract GSUB scripts
        let gsub_scripts = info
            .font
            .gsub()
            .and_then(|gsub| gsub.script_list())
            .map(|script_list| script_list.script_records())
            .into_iter()
            .flatten()
            .map(|script| script.script_tag().to_string());
        // Extract GPOS scripts
        let gpos_scripts = info
            .font
            .gpos()
            .and_then(|gpos| gpos.script_list())
            .map(|script_list| script_list.script_records())
            .into_iter()
            .flatten()
            .map(|script| script.script_tag().to_string());
        gsub_scripts.chain(gpos_scripts)
    }
}

impl FontMatcher for ScriptsMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let all_scripts: HashSet<String> = self.extract(info).collect();
        self.wanted_scripts
            .iter()
            .all(|script| all_scripts.contains(script))
    }
}

/// Matcher for font tables
pub(crate) struct TablesMatcher {
    wanted_tables: Vec<Tag>,
}

impl TablesMatcher {
    /// Create a new tables matcher
    pub fn new(tables: &[Tag]) -> Self {
        Self {
            wanted_tables: tables.to_vec(),
        }
    }

    /// Extract font tables from a font
    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = String> + 'a {
        info.font
            .table_directory
            .table_records()
            .iter()
            .map(|record| record.tag().to_string())
    }
}

impl FontMatcher for TablesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let all_tables: HashSet<String> = self.extract(info).collect();
        self.wanted_tables
            .iter()
            .all(|table| all_tables.contains(&table.to_string()))
    }
}

/// Matcher for Unicode codepoints
pub(crate) struct CodepointsMatcher {
    codepoints: Vec<char>,
}

impl CodepointsMatcher {
    /// Create a new codepoints matcher
    pub fn new(codepoints: &[char]) -> Self {
        Self {
            codepoints: codepoints.to_vec(),
        }
    }

    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = char> + 'a {
        info.font
            .charmap()
            .mappings()
            .flat_map(|(codepoint, _)| char::try_from(codepoint))
    }
}

impl FontMatcher for CodepointsMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let charset: HashSet<char> = self.extract(info).collect();
        self.codepoints.iter().all(|cp| charset.contains(cp))
    }
}

/// Matcher for font names
pub(crate) struct NameMatcher {
    patterns: Vec<regex::Regex>,
}

impl NameMatcher {
    /// Create a new name matcher
    pub fn new(patterns: &[regex::Regex]) -> Self {
        Self {
            patterns: patterns.to_vec(),
        }
    }

    fn extract<'a>(&self, info: &'a FontInfo) -> impl Iterator<Item = String> + 'a {
        if let Ok(name) = info.font.name() {
            Either::Left(
                name.name_record()
                    .iter()
                    .flat_map(move |record| record.string(name.string_data()))
                    .map(|n| n.to_string()),
            )
        } else {
            Either::Right(std::iter::empty())
        }
    }
}

impl FontMatcher for NameMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        // We don't join them all into one string here, because then ^$ won't work
        let all_names: Vec<String> = self.extract(info).collect();
        self.patterns
            .iter()
            .any(|pattern| all_names.iter().any(|name| pattern.is_match(name)))
    }
}
