use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAddress {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressCommand {
    Cell(CellAddress),
    Range {
        start: CellAddress,
        end: CellAddress,
    },
    Write {
        target: CellAddress,
        value: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AddressError {
    #[error("invalid address: {0}")]
    InvalidAddress(String),
}

pub type Result<T> = std::result::Result<T, AddressError>;

pub fn parse_address_command(input: &str) -> Result<AddressCommand> {
    if let Some((lhs, rhs)) = input.split_once('=') {
        let target = parse_cell_address(lhs.trim())?;
        return Ok(AddressCommand::Write {
            target,
            value: rhs.to_string(),
        });
    }

    if let Some((lhs, rhs)) = input.split_once(':') {
        let start = parse_cell_address(lhs.trim())?;
        let end = parse_cell_address(rhs.trim())?;
        return Ok(AddressCommand::Range {
            start: normalize_cell_range(start, end).0,
            end: normalize_cell_range(start, end).1,
        });
    }

    Ok(AddressCommand::Cell(parse_cell_address(input.trim())?))
}

pub fn parse_cell_address(input: &str) -> Result<CellAddress> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AddressError::InvalidAddress("empty address".to_string()));
    }

    let split_at = trimmed
        .find(|ch: char| ch.is_ascii_digit())
        .ok_or_else(|| AddressError::InvalidAddress(trimmed.to_string()))?;
    let (col_part, row_part) = trimmed.split_at(split_at);

    if col_part.is_empty()
        || row_part.is_empty()
        || !col_part.chars().all(|ch| ch.is_ascii_alphabetic())
        || !row_part.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(AddressError::InvalidAddress(trimmed.to_string()));
    }

    let col = col_letter_to_index(col_part)?;
    let row_number = row_part
        .parse::<usize>()
        .map_err(|_| AddressError::InvalidAddress(trimmed.to_string()))?;
    let row = row_number
        .checked_sub(1)
        .ok_or_else(|| AddressError::InvalidAddress(trimmed.to_string()))?;

    Ok(CellAddress { row, col })
}

pub fn col_letter_to_index(input: &str) -> Result<usize> {
    let trimmed = input.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err(AddressError::InvalidAddress(input.to_string()));
    }

    let mut value = 0usize;
    for ch in trimmed.chars() {
        let letter = ch.to_ascii_uppercase();
        let digit = (letter as u8 - b'A' + 1) as usize;
        value = value
            .checked_mul(26)
            .and_then(|current| current.checked_add(digit))
            .ok_or_else(|| AddressError::InvalidAddress(input.to_string()))?;
    }

    value
        .checked_sub(1)
        .ok_or_else(|| AddressError::InvalidAddress(input.to_string()))
}

pub fn index_to_col_letters(index: usize) -> String {
    let mut n = index + 1;
    let mut out = Vec::new();

    while n > 0 {
        let rem = (n - 1) % 26;
        out.push((b'A' + rem as u8) as char);
        n = (n - 1) / 26;
    }

    out.iter().rev().collect()
}

fn normalize_cell_range(a: CellAddress, b: CellAddress) -> (CellAddress, CellAddress) {
    (
        CellAddress {
            row: a.row.min(b.row),
            col: a.col.min(b.col),
        },
        CellAddress {
            row: a.row.max(b.row),
            col: a.col.max(b.col),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_parse_single_cell() {
        assert_eq!(
            parse_address_command("B9").unwrap(),
            AddressCommand::Cell(CellAddress { row: 8, col: 1 })
        );
    }

    #[test]
    fn test_address_parse_range() {
        assert_eq!(
            parse_address_command("B1:B3").unwrap(),
            AddressCommand::Range {
                start: CellAddress { row: 0, col: 1 },
                end: CellAddress { row: 2, col: 1 },
            }
        );
    }

    #[test]
    fn test_address_parse_write() {
        assert_eq!(
            parse_address_command("B7=10").unwrap(),
            AddressCommand::Write {
                target: CellAddress { row: 6, col: 1 },
                value: "10".to_string(),
            }
        );
    }

    #[test]
    fn test_address_parse_aa_column() {
        assert_eq!(
            parse_cell_address("AA1").unwrap(),
            CellAddress { row: 0, col: 26 }
        );
    }

    #[test]
    fn test_address_rejects_invalid_row() {
        assert!(parse_cell_address("A0").is_err());
    }

    #[test]
    fn test_address_rejects_invalid_format() {
        assert!(parse_address_command("9B").is_err());
    }

    #[test]
    fn test_address_index_to_col_letters() {
        assert_eq!(index_to_col_letters(0), "A");
        assert_eq!(index_to_col_letters(25), "Z");
        assert_eq!(index_to_col_letters(26), "AA");
    }
}
