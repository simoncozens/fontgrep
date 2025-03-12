// this_file: fontgrep/src/cli.rs
//
// Command-line interface for fontgrep

use crate::{font::FontInfo, query::FontQuery, FontgrepError, Result};
use clap::{Args as ClapArgs, Parser, Subcommand};
use regex::Regex;
use skrifa::Tag;
use std::path::PathBuf;

/// Command-line arguments for fontgrep
#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to search for fonts based on various criteria",
    long_about = "fontgrep is a command-line tool that helps you find fonts based on their properties, such as OpenType features, variation axes, scripts, and more. It can search through directories of font files and maintain a cache for faster subsequent searches."
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(
        short,
        long,
        help = "Enable verbose output",
        long_help = "Enable verbose output mode that shows additional information \
                    about the search process and font properties."
    )]
    pub verbose: bool,

    /// Output as JSON
    #[arg(
        short,
        long,
        help = "Output as JSON",
        long_help = "Output results in JSON format for machine processing. \
                    If not specified, results are output as human-readable text."
    )]
    pub json: bool,
}

/// Subcommands for fontgrep
#[derive(Subcommand, Debug)]
enum Commands {
    /// Find fonts based on various criteria (without cache)
    Find(SearchArgs),

    /// Show information about a font
    Font(InfoArgs),
}

/// Arguments for the search command
#[derive(ClapArgs, Debug)]
pub(crate) struct SearchArgs {
    /// Directories or font files to search
    #[arg(
        required = true,
        help = "Directories or font files to search",
        long_help = "One or more directories or font files to search. \
                    Directories will be searched recursively for font files."
    )]
    pub paths: Vec<PathBuf>,

    /// Variation axes to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Variation axes to search for (e.g., wght,wdth)",
        long_help = "Comma-separated list of OpenType variation axes to search for. \
                    Common axes include:\n\
                    - wght: Weight\n\
                    - wdth: Width\n\
                    - ital: Italic\n\
                    - slnt: Slant\n\
                    - opsz: Optical Size"
    )]
    pub axes: Vec<String>,

    /// OpenType features to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "OpenType features to search for (e.g., smcp,onum)",
        long_help = "Comma-separated list of OpenType features to search for. \
                    Common features include:\n\
                    - smcp: Small Capitals\n\
                    - onum: Oldstyle Numerals\n\
                    - liga: Standard Ligatures\n\
                    - kern: Kerning\n\
                    - dlig: Discretionary Ligatures"
    )]
    pub features: Vec<String>,

    /// OpenType scripts to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "OpenType scripts to search for (e.g., latn,cyrl)",
        long_help = "Comma-separated list of OpenType script tags to search for. \
                    Common scripts include:\n\
                    - latn: Latin\n\
                    - cyrl: Cyrillic\n\
                    - grek: Greek\n\
                    - arab: Arabic\n\
                    - deva: Devanagari"
    )]
    pub scripts: Vec<String>,

    /// Font tables to search for
    #[arg(
        short = 'T',
        long,
        value_delimiter = ',',
        help = "Font tables to search for (e.g., GPOS,GSUB)",
        long_help = "Comma-separated list of OpenType table tags to search for. \
                    Common tables include:\n\
                    - GPOS: Glyph Positioning\n\
                    - GSUB: Glyph Substitution\n\
                    - GDEF: Glyph Definition\n\
                    - BASE: Baseline\n\
                    - OS/2: OS/2 and Windows Metrics"
    )]
    pub tables: Vec<Tag>,

    /// Only show variable fonts
    #[arg(
        short,
        long,
        help = "Only show variable fonts",
        long_help = "Only show variable fonts that support OpenType Font Variations."
    )]
    pub variable: bool,

    /// Regular expressions to match against font names
    #[arg(
        short,
        long,
        help = "Regular expressions to match against font names",
        long_help = "One or more regular expressions to match against font names. \
                    The search is case-insensitive and matches anywhere in the name."
    )]
    pub name: Vec<Regex>,

    /// Unicode codepoints or ranges to search for
    #[arg(
        short = 'u',
        long,
        value_parser = parse_codepoints,
        help = "Unicode codepoints or ranges to search for (e.g., U+0041-U+005A,U+0061)",
        long_help = "Comma-separated list of Unicode codepoints or ranges to search for. \
                    Formats accepted:\n\
                    - Single codepoint: U+0041 or 0041\n\
                    - Range: U+0041-U+005A\n\
                    - Single character: A"
    )]
    pub codepoints: Vec<char>,

    /// Text to check for support
    #[arg(
        short,
        long,
        help = "Text to check for support",
        long_help = "Text string to check for font support. \
                    All characters in the text must be supported by the font."
    )]
    pub text: Option<String>,

    /// Number of parallel jobs to use
    #[arg(
        short,
        long,
        default_value_t = num_cpus::get(),
        help = "Number of parallel jobs to use",
        long_help = "Number of parallel jobs to use for searching and processing fonts. \
                    Defaults to the number of CPU cores available."
    )]
    pub jobs: usize,
}

/// Arguments for the info command
#[derive(ClapArgs, Debug)]
struct InfoArgs {
    /// Font file to show information about
    #[arg(required = true)]
    pub path: PathBuf,

    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,
}

/// Execute the command
pub fn execute(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Find(args) => {
            let query = FontQuery::from(args);
            let results = query.execute()?;

            // Output results
            output_results(&results, cli.json)?;
        }
        Commands::Font(args) => {
            // Load font
            let font_info = FontInfo::load(&args.path)?;

            // Output font info
            output_font_info(&font_info, args.detailed, cli.json)?;
        }
    }

    Ok(())
}

/// Parse codepoints from strings
pub fn parse_codepoints(input: &str) -> Result<Vec<char>> {
    let mut result = Vec::new();

    for item in input.split(",") {
        if item.contains('-') {
            // Parse range
            let parts: Vec<&str> = item.split('-').collect();
            if parts.len() != 2 {
                return Err(FontgrepError::Parse(format!(
                    "Invalid codepoint range: {}",
                    item
                )));
            }

            let start = parse_codepoint(parts[0])?;
            let end = parse_codepoint(parts[1])?;

            let start_u32 = start as u32;
            let end_u32 = end as u32;

            if start_u32 > end_u32 {
                return Err(FontgrepError::Parse(format!(
                    "Invalid codepoint range: {} > {}",
                    start_u32, end_u32
                )));
            }

            for cp in start_u32..=end_u32 {
                if let Some(c) = char::from_u32(cp) {
                    result.push(c);
                }
            }
        } else {
            // Parse single codepoint
            result.push(parse_codepoint(item)?);
        }
    }

    Ok(result)
}

/// Parse a single codepoint from a string
fn parse_codepoint(input: &str) -> Result<char> {
    if input.len() == 1 {
        // Single character
        return Ok(input.chars().next().unwrap());
    }

    let input = input.trim_start_matches("U+").trim_start_matches("u+");
    let cp = u32::from_str_radix(input, 16)
        .map_err(|_| FontgrepError::Parse(format!("Invalid codepoint: {}", input)))?;

    char::from_u32(cp)
        .ok_or_else(|| FontgrepError::Parse(format!("Invalid Unicode codepoint: U+{:04X}", cp)))
}

/// Output results
fn output_results(results: &[String], json_output: bool) -> Result<()> {
    if json_output {
        let json = serde_json::to_string_pretty(results)?;
        println!("{}", json);
    } else {
        for result in results {
            println!("{}", result);
        }
    }

    Ok(())
}

/// Output font info
fn output_font_info(info: &FontInfo, detailed: bool, json_output: bool) -> Result<()> {
    if json_output {
        let json = serde_json::to_string_pretty(info)?;
        println!("{}", json);
    } else {
        println!("Name: {}", info.name_string);
        println!("Variable: {}", info.is_variable);

        if detailed {
            println!("Axes: {}", info.axes.join(", "));
            println!("Features: {}", info.features.join(", "));
            println!("Scripts: {}", info.scripts.join(", "));
            println!("Tables: {}", info.tables.join(", "));
            println!("Charset: {}", info.charset_string);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codepoint() {
        assert_eq!(parse_codepoint("A").unwrap(), 'A');
        assert_eq!(parse_codepoint("0041").unwrap(), 'A');
        assert_eq!(parse_codepoint("U+0041").unwrap(), 'A');
        assert_eq!(parse_codepoint("u+0041").unwrap(), 'A');
    }

    #[test]
    fn test_parse_codepoints() {
        let input = "A,U+0042-U+0044";
        let result = parse_codepoints(input).unwrap();
        assert_eq!(result, vec!['A', 'B', 'C', 'D']);
    }
}
