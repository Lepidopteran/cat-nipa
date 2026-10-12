# Cat Nipa

Pseudo alternative to [nipa](https://github.com/Wilhansen/nipa) made in Rust.

Cat Nipa is a utility for decryption, decompression, and extraction of Nippon Ichi Software (NIS) archive files (`.npa`). Supports both command-line and graphical interfaces.

## Motivation

**Why the name?** I like cats, and when thinking of a name for this project and trying the project [nipa](https://github.com/Wilhansen/nipa), I thought of catnip.

**Why make this?** I couldn't use [nipa](https://github.com/Wilhansen/nipa) easily on Linux, so I decided to see if I could make a Rust alternative.

## Features

- **Dual Interface**: CLI for batch extraction and GUI for interactive exploration
- **Multi-Game Support**: Handles archives from various NIS visual novels
- **Parallel Processing**: Uses Rayon for efficient multi-threaded extraction
- **Image Viewer**: Built-in zoomable/pannable image viewer in the GUI
- **Auto-Detection**: Automatically detects the correct decryption key when multiple games are supported

## Building

### Requirements

- Rust 1.75+
- A C toolchain (for some dependencies)

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

The binary will be available at `target/release/cat-nipa`.

## Usage

### Command-Line Interface

```bash
# Extract all files from an archive
cat-nipa -i <input.npa> -g <GAME> -o <output-dir>

# Interactive game selection
cat-nipa -i <input.npa> -g <GAME> -o <output-dir>

# GUI mode
cat-nipa --gui
```

### Available Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input` | `-i` | Input archive file | - |
| `--game` | `-g` | Game to use for decryption | `ChaosHead` |
| `--output` | `-o` | Output directory | `<filename>` |
| `--gui` | | Launch GUI instead of CLI | `false` |

### Supported Games

The following Nippon Ichi Software games are currently supported:

| Game | Game Enum Value |
|------|-----------------|
| Chaos;Head | `ChaosHead` |
| Chaos;Head (Trial 1) | `ChaosHeadTrailOne` |
| Chaos;Head (Trial 2) | `ChaosHeadTrailTwo` |
| Muramasa | `Muramasa` |
| Muramasa (Trial) | `MuramasaTrail` |
| Muramasa: The Demon Blade | `MuramasaAD` |
| Muramasa: Sign of the Soul | `MuramasaSS` |
| Sumaga | `Sumaga` |
| Sumaga (Special) | `SumagaSpecial` |
| Sumaga 3% Trial | `SumagaThreePercent` |
| Django | `Django` |
| Django Trial | `DjangoTrial` |
| Lamento -Beyond the Void- | `Lamento` |
| Lamento (Trial) | `LamentoTrail` |
| SweetPool | `SweetPool` |
| Demonbane: The Best | `Demonbane` |
| Axanael Trial | `Axanael` |
| Kikokugai (N2 System) | `Kikokugai` |
| Sonicomi Trial 2 | `SonicomiTrialTwo` |
| Sonicomi | `Sonicomi` |
| Guilty Crown: Lost Christmas | `LostX` |
| Guilty Crown: Lost Christmas (Trailer) | `LostXTrailer` |
| Dramatical Murder | `DramaticalMurder` |
| Dramatical Murder Re:Connect | `DramaticalMurderReConnect` |
| Totono | `Totono` |

### GUI Mode

The GUI provides:
- **File Tree**: Browse the archive structure
- **Image Preview**: Click on images to view them with zoom and pan controls
- **Auto-Game Detection**: Automatically tries to find the correct game key

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `egui` | Immediate mode GUI framework |
| `eframe` | Application framework for egui |
| `clap` | Command-line argument parsing |
| `flate2` | zlib/gzip decompression |
| `rayon` | Data-parallel processing |
| `image` | Image loading and processing |
| `egui_ltreeview` | Hierarchical tree view |

## Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd cat-nipa

# Build
cargo build --release

# Run tests
cargo test
```

## Contributing

Contributions are welcome! Please note:

1. This project is for educational purposes only
2. Do not include original `.npa` files in the repository
3. Add new game support by extending the `Game` enum and adding decryption keys
4. Follow Rust best practices and maintain code style

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is provided for archival and preservation purposes. Please ensure you have the right to use any files you extract.

## Credits & Acknowledgments

### Original NPA Format Documentation
This project is based on the reverse engineering work done by **Minagi** from the [NovelNews.net forum](http://forums.novelnews.net/showthread.php?t=35621) for the original [**nipa** tool](https://github.com/Wilhansen/nipa).

The NPA format documentation was originally created and shared by Minagi. Key resources:
- [Original NPA Format Documentation](http://www.tsukuru.info/nipa/npadoc.html)
- [Minagi's Forum Post](http://forums.novelnews.net/showthread.php?t=35621)
- [Wilhansen/nipa GitHub Repository](https://github.com/Wilhansen/nipa)

### Original NPA Tool Author
The original C implementation (nipa.c) and header (nipa.h) were created by **Minagi**, who reverse-engineered the NPA archive format used by Nitro+ and Nitro+ CHiRAL games.

### Additional Resources
- [Nipal Documentation](http://www.tsukuru.info/nipa/) - Archived documentation of the NPA format
- [NovelNews Forum](http://forums.novelnews.net/) - Where the original reverse engineering work was discussed

### Libraries Used
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - Application framework for egui
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [flate2](https://github.com/alexcrichton/flate2-rs) - zlib/gzip decompression (includes zlib port)
- [rayon](https://github.com/rayon-rs/rayon) - Data-parallel processing
- [image](https://github.com/rust-lang/image) - Image loading and processing
- [egui_ltreeview](https://github.com/emilk/egui/tree/master/crates/egui_ltreeview) - Hierarchical tree view
