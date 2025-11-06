# fontgrep

A command-line tool for searching and analyzing font files based on various criteria.

## Features

- Search for fonts based on:
  - OpenType variation axes (e.g., weight, width)
  - OpenType features (e.g., small caps, old-style numerals)
  - OpenType scripts (e.g., Latin, Cyrillic)
  - Font tables (e.g., GPOS, GSUB)
  - Unicode character support
  - Font name patterns
- Progressive output for immediate feedback
- Parallel processing for improved performance
- Output in text or JSON format

## Installation

```bash
cargo install --git https://github.com/simoncozens/fontgrep
```

## Usage

### Basic Font Searching

```bash
# Find variable fonts
fontgrep --variable /path/to/fonts

# Find fonts with specific features
fontgrep -f smcp,onum /path/to/fonts

# Find fonts supporting specific scripts
fontgrep -s latn,cyrl /path/to/fonts

# Find fonts by name pattern
fontgrep -n "Roboto.*Mono" /path/to/fonts

# Find fonts supporting specific Unicode ranges
fontgrep -u "U+0041-U+005A,U+0061-U+007A" /path/to/fonts

# Find fonts with specific tables
fontgrep -T GPOS,GSUB /path/to/fonts

# Find fonts supporting specific text
fontgrep -t "Hello World" /path/to/fonts
```

### Combining Search Criteria

```bash
# Find variable fonts with small caps feature
fontgrep --variable -f smcp /path/to/fonts

# Find fonts supporting both Latin and Cyrillic scripts
fontgrep -s latn,cyrl /path/to/fonts

# Find fonts with specific name pattern and features
fontgrep -n "Roboto" -f liga,kern /path/to/fonts
```

### Output Formats

```bash
# Output in JSON format
fontgrep -j /path/to/fonts

# Combine JSON output with search criteria
fontgrep -j -f smcp,onum /path/to/fonts
```

## Command-Line Options

- `-a, --axes <AXES>`: Comma-separated list of OpenType variation axes to search for (e.g., wght,wdth)
- `-f, --features <FEATURES>`: Comma-separated list of OpenType features to search for (e.g., smcp,onum)
- `-s, --scripts <SCRIPTS>`: Comma-separated list of OpenType script tags to search for (e.g., latn,cyrl)
- `-T, --tables <TABLES>`: Comma-separated list of OpenType table tags to search for (e.g., GPOS,GSUB)
- `-v, --variable`: Only show variable fonts that support OpenType Font Variations
- `-n, --name <NAME>`: Regular expressions to match against font names
- `-u, --codepoints <CODEPOINTS>`: Unicode codepoints or ranges to search for (e.g., U+0041-U+005A,U+0061)
- `-t, --text <TEXT>`: Text string to check for support
- `-J, --jobs <JOBS>`: Number of parallel jobs to use (defaults to CPU core count)
- `--verbose`: Enable verbose output
- `-j, --json`: Output results in JSON format
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Performance

- Uses memory mapping for efficient font file access
- Employs parallel processing for searching and font analysis
- Provides progressive output for immediate feedback

## Development

### Building from Source

```bash
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [skrifa](https://github.com/googlefonts/skrifa) for font parsing
- Uses [clap](https://github.com/clap-rs/clap) for command-line argument parsing
- Uses [jwalk](https://github.com/jessegrosjean/jwalk) for parallel directory traversal 
