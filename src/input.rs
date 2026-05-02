//! Input mode management for zellij-sheets.
//!
//! This module defines the different modes the spreadsheet viewer can be in,
//! such as Normal, Search, or Pending (waiting for a second key in a sequence).
//!
//! By centralizing these types, we ensure behavior parity between the native TUI
//! and the Zellij WASM plugin.

/// Input mode — shared between the Native TUI and the Zellij Plugin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    /// A prefix key has been pressed and we are waiting for the second key (e.g., 'g').
    Pending(PendingKey),
}

/// First key of a two-key sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingKey {
    /// `g` was pressed; waiting for a second `g` to go-to-top.
    LowercaseG,
}
