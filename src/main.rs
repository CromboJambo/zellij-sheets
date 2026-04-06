#[cfg(target_family = "wasm")]
mod plugin;

#[cfg(not(target_family = "wasm"))]
use std::io::{self, IsTerminal, Read};
#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use anyhow::{anyhow, bail, Context};
#[cfg(not(target_family = "wasm"))]
use zellij_sheets::data_loader::{DataSource, LoadedData};
#[cfg(not(target_family = "wasm"))]
use zellij_sheets::{
    index_to_col_letters, load_csv_from_reader, load_data, parse_address_command, write_csv,
    AddressCommand, CellAddress, SheetsConfig, SheetsState, UiRenderer,
};

#[cfg(not(target_family = "wasm"))]
enum InputSource {
    Path(PathBuf),
    Stdin,
}

#[cfg(not(target_family = "wasm"))]
struct SheetsArgs {
    input_source: InputSource,
    address: Option<AddressCommand>,
}

#[cfg(not(target_family = "wasm"))]
const CLI_USAGE: &str = "\
Usage:
  zellij-sheets [file|-] [address]
  zellij-sheets --input <file> [address]

Examples:
  zellij-sheets data.csv
  zellij-sheets data.csv B9
  zellij-sheets data.csv B1:B3
  zellij-sheets data.csv B7=10
  cat data.csv | zellij-sheets B2";

#[cfg(not(target_family = "wasm"))]
fn main() -> anyhow::Result<()> {
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => return Ok(()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    match args.address {
        Some(address) => execute_cli_action(args.input_source, address),
        None => render_cli(args.input_source),
    }
}

#[cfg(not(target_family = "wasm"))]
fn render_cli(input_source: InputSource) -> anyhow::Result<()> {
    let config = SheetsConfig::default();
    let mut state = SheetsState::new(Arc::new(config));

    match input_source {
        InputSource::Path(path) => state.load_file(path)?,
        InputSource::Stdin => {
            let data = read_stdin_csv()?;
            state.init(data)?;
        }
    }

    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100);
    let height = std::env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(24);
    state.resize(width, height);

    let renderer = UiRenderer::new();
    println!(
        "{}",
        renderer
            .draw_ui(&state)
            .unwrap_or_else(|error| format!("Error: {error}"))
    );
    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn execute_cli_action(input_source: InputSource, address: AddressCommand) -> anyhow::Result<()> {
    match input_source {
        InputSource::Path(path) => {
            let mut data = load_data(&path)?;
            execute_address_command(&mut data, &address, Some(&path))?;
            Ok(())
        }
        InputSource::Stdin => {
            let mut data = read_stdin_csv()?;
            execute_address_command(&mut data, &address, None)?;
            Ok(())
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn execute_address_command(
    data: &mut LoadedData,
    command: &AddressCommand,
    path: Option<&PathBuf>,
) -> anyhow::Result<()> {
    match command {
        AddressCommand::Cell(cell) => {
            let value = get_addressed_cell(data, *cell)?;
            println!("{value}");
        }
        AddressCommand::Range { start, end } => {
            for row in start.row..=end.row {
                for col in start.col..=end.col {
                    println!("{}", get_addressed_cell(data, CellAddress { row, col })?);
                }
            }
        }
        AddressCommand::Write { target, value } => {
            let path = path.ok_or_else(|| anyhow!("cannot write CSV updates back to stdin"))?;
            if data.source != DataSource::Csv {
                bail!("cell writes are only supported for CSV inputs");
            }

            let row_count = data.rows.len();
            let col_count = data.headers.len();
            if target.col >= col_count {
                bail!(
                    "{} is out of bounds for {} columns",
                    format_cell_address(*target),
                    col_count
                );
            }
            let row = data.rows.get_mut(target.row).ok_or_else(|| {
                anyhow!(
                    "{} is out of bounds for {} data rows",
                    format_cell_address(*target),
                    row_count
                )
            })?;
            let cell = row
                .get_mut(target.col)
                .ok_or_else(|| anyhow!("{} is out of bounds", format_cell_address(*target)))?;
            *cell = value.clone();
            write_csv(path, data)?;
            println!("{value}");
        }
    }

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn get_addressed_cell(data: &LoadedData, cell: CellAddress) -> anyhow::Result<String> {
    if cell.col >= data.headers.len() {
        bail!(
            "{} is out of bounds for {} columns",
            format_cell_address(cell),
            data.headers.len()
        );
    }

    data.rows
        .get(cell.row)
        .and_then(|row| row.get(cell.col))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "{} is out of bounds for {} data rows",
                format_cell_address(cell),
                data.rows.len()
            )
        })
}

#[cfg(not(target_family = "wasm"))]
fn read_stdin_csv() -> anyhow::Result<LoadedData> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("failed to read CSV from stdin")?;
    load_csv_from_reader(buffer.as_bytes()).context("failed to parse CSV from stdin")
}

#[cfg(not(target_family = "wasm"))]
fn parse_args() -> anyhow::Result<Option<SheetsArgs>> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut input_path: Option<PathBuf> = None;
    let mut positional = Vec::new();
    let mut index = 0;

    while index < raw_args.len() {
        if matches!(raw_args[index].as_str(), "-h" | "--help") {
            println!("{CLI_USAGE}");
            return Ok(None);
        } else if matches!(raw_args[index].as_str(), "-V" | "--version") {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(None);
        } else if raw_args[index] == "--input" {
            let value = raw_args
                .get(index + 1)
                .ok_or_else(|| anyhow!("missing value for --input"))?;
            input_path = Some(PathBuf::from(value));
            index += 2;
        } else {
            positional.push(raw_args[index].clone());
            index += 1;
        }
    }

    let stdin_has_data = !io::stdin().is_terminal();
    let had_input_flag = input_path.is_some();
    let input_source = if let Some(path) = input_path {
        InputSource::Path(path)
    } else if let Some(first) = positional.first() {
        if first == "-" {
            InputSource::Stdin
        } else if positional.len() == 1
            && stdin_has_data
            && std::fs::metadata(first).is_err()
            && parse_address_command(first).is_ok()
        {
            InputSource::Stdin
        } else {
            InputSource::Path(PathBuf::from(first))
        }
    } else if stdin_has_data {
        InputSource::Stdin
    } else {
        bail!("{CLI_USAGE}");
    };

    let address_arg = match &input_source {
        InputSource::Path(_) => positional.get(1),
        InputSource::Stdin => {
            if positional.first().is_some_and(|value| value != "-") {
                positional.first()
            } else {
                positional.get(1)
            }
        }
    };

    if positional.len() > 2 || (had_input_flag && positional.len() > 1) {
        bail!("too many arguments\n\n{CLI_USAGE}");
    }

    let address = address_arg
        .map(|value| parse_address_command(value))
        .transpose()
        .map_err(|error| anyhow!("{error}"))?;

    Ok(Some(SheetsArgs {
        input_source,
        address,
    }))
}

#[cfg(not(target_family = "wasm"))]
fn format_cell_address(cell: CellAddress) -> String {
    format!("{}{}", index_to_col_letters(cell.col), cell.row + 1)
}

#[cfg(target_family = "wasm")]
use plugin::PluginState;
#[cfg(target_family = "wasm")]
use zellij_tile::{shim::report_panic, ZellijPlugin};

#[cfg(target_family = "wasm")]
zellij_tile::register_plugin!(PluginState);
