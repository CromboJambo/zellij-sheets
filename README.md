# Zellij Sheets

A terminal-based spreadsheet viewer powered by Zellij's TUI capabilities. Built for users who want to view and navigate spreadsheet data directly in their terminal with a modern, keyboard-driven interface.

## Features

- **Multi-format Support**: View CSV, Excel (.xlsx), and Parquet files
- **Keyboard Navigation**: Navigate through data using arrow keys, page up/down, and home/end
- **Selection System**: Highlight and focus on specific rows
- **Configurable Theme**: Customize colors and display settings
- **Responsive Layout**: Adapts to terminal window size
- **Data Type Detection**: Automatically identifies different data types
- **Performance**: Uses Polars for efficient data loading and manipulation

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [Zellij](https://github.com/zellij-org/zellij) (recommended)
- [Zellij plugin system](https://github.com/zellij-org/zellij-plugin-api)

### Building from Source

```bash
cd zellij-sheets
cargo build --release
```

### Using as a Zellij Plugin

1. **Copy the binary** to your Zellij plugins directory:
```bash
cp target/release/zellij-sheets ~/.local/share/zellij/plugins/
```

2. **Add to your Zellij config** (`~/.config/zellij/config.kdl`):

```kdl
plugins {
    sheets = "/home/youruser/.local/share/zellij/plugins/zellij-sheets"
}
```

3. **Create a layout** (`layouts/spreadsheet.kdl`):

```kdl
layout "spreadsheet" {
    default_tab layout {
        pane size=1:1 split_direction="Right" {
            pane size=1:3 split_direction="Down" {
                spawn "zellij" "plugin" "zellij-sheets" "--input" "/path/to/your/data.csv"
            }
            pane size=1:1 {
                # Other panes
            }
        }
    }
}
```

## Usage

### Command Line

```bash
# View a CSV file
zellij-sheets --input data.csv

# View an Excel file
zellij-sheets --input spreadsheet.xlsx

# View a Parquet file
zellij-sheets --input data.parquet

# Set preview row count
zellij-sheets --input data.csv --preview 50
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Arrow Up` | Select previous row |
| `Arrow Down` | Select next row |
| `Arrow Left` | Scroll left (if needed) |
| `Arrow Right` | Scroll right (if needed) |
| `Home` | Go to top of data |
| `End` | Go to bottom of data |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `q` or `Ctrl+C` | Quit |
| `/` | Search (coming soon) |

## Configuration

Create a `config.toml` file in `~/.config/zellij-sheets/` to customize the appearance and behavior:

```toml
[theme]
background = "#000000"
text = "#FFFFFF"
header_background = "#0055AA"
selected_background = "#00AAFF"

[display]
preview_rows = 20
truncate_long_values = true
max_cell_length = 50

[behavior]
page_size = 10
auto_refresh = false
```

## Project Structure

```
zellij-sheets/
├── src/
│   ├── main.rs          # Entry point and plugin implementation
│   ├── lib.rs           # Library exports
│   ├── config.rs        # Configuration management
│   ├── data_loader.rs   # Data loading logic
│   ├── state.rs         # Application state
│   └── ui.rs            # UI rendering helpers
├── config/
│   └── config.toml      # Default configuration
├── layouts/             # Zellij layout examples
├── README.md
└── Cargo.toml
```

## Supported File Formats

- **CSV**: Comma-separated values (UTF-8)
- **Excel**: .xlsx and .xls files (requires calamine)
- **Parquet**: Columnar storage format (requires polars)

## Architecture

### Core Components

1. **Data Loading Module** (`data_loader.rs`)
   - Handles file format detection and parsing
   - Uses Polars for efficient data operations
   - Supports multiple input formats

2. **State Management** (`state.rs`)
   - Manages application state and data
   - Handles user input and navigation
   - Maintains scroll and selection positions

3. **UI Rendering** (`ui.rs`)
   - Draws header, grid, and footer
   - Applies styling and themes
   - Handles responsive layout

4. **Configuration** (`config.rs`)
   - Manages user preferences
   - Validates configuration
   - Provides sensible defaults

### Plugin Architecture

The project uses Zellij's plugin API to integrate seamlessly with Zellij's terminal multiplexer:

- **Event Handling**: Responds to Zellij events (Init, Resize, Input, Draw, Quit)
- **State Management**: Maintains application state across events
- **UI Rendering**: Draws to Zellij's screen buffer

## Development

### Running Tests

```bash
cargo test
```

### Adding New Features

1. Add new functionality to the appropriate module
2. Update state management if needed
3. Add UI rendering in `ui.rs`
4. Test thoroughly with different file formats

### Code Style

- Follow Rust conventions
- Use descriptive names
- Add comments for complex logic
- Maintain error handling

## Future Enhancements

- [ ] Search functionality
- [ ] Column filtering
- [ ] Data export capabilities
- [ ] Custom column widths
- [ ] Color themes
- [ ] Performance optimizations for large files
- [ ] Real-time file watching
- [ ] Advanced filtering and sorting
- [ ] Cell value editing

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Credits

Built with:
- [Zellij](https://github.com/zellij-org/zellij) - Terminal multiplexer
- [Polars](https://github.com/pola-rs/polars) - Data processing framework
- [Calamine](https://github.com/tafia/calamine) - Excel/CSV reading
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI (planned)

## Acknowledgments

Inspired by the need for a lightweight, terminal-based spreadsheet viewer that integrates seamlessly with modern terminal workflows.

---

**Note**: This is an early-stage project. Features marked as "coming soon" are planned but not yet implemented.