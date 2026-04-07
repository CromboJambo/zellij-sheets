//! Native interactive TUI for zellij-sheets.
//!
//! Owns the raw-mode event loop for the native binary. Responsibilities:
//!
//! - Enter/leave raw mode and the alternate screen.
//! - Poll crossterm events and translate them into `SheetsState` method calls.
//! - Re-render via `UiRenderer` after every state-changing event.
//! - Translate terminal resize events into `SheetsState::resize` calls.
//!
//! Key mapping mirrors the plugin (`plugin.rs`) exactly. All navigation logic
//! lives in `SheetsState`; this module is a thin keybinding layer.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};

use crate::state::{SearchDirection, SheetsState};
use crate::ui::UiRenderer;

/// Input mode — mirrors `plugin.rs` `InputMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Search,
    /// A prefix key has been pressed and we are waiting for the second key.
    Pending(PendingKey),
}

/// First key of a two-key sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingKey {
    /// `g` was pressed; waiting for a second `g` to go-to-top.
    LowercaseG,
}

/// Run the interactive TUI until the user quits.
///
/// Enters raw mode and the alternate screen, then loops on crossterm events.
/// On exit (any path) the terminal is restored to its original state.
pub fn run(state: &mut SheetsState) -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = event_loop(state, &mut stdout);

    // Always restore the terminal, even if the event loop returned an error.
    let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);
    let _ = terminal::disable_raw_mode();

    result
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

fn event_loop(state: &mut SheetsState, stdout: &mut impl Write) -> anyhow::Result<()> {
    let renderer = UiRenderer::new();
    let mut input_mode = InputMode::default();

    // Sync initial terminal size.
    let (cols, rows) = terminal::size()?;
    state.resize(cols as usize, rows as usize);

    // Draw first frame before waiting for any input.
    render(state, &renderer, stdout)?;

    loop {
        let ev = event::read()?;

        match ev {
            // ----------------------------------------------------------------
            // Terminal resize
            // ----------------------------------------------------------------
            Event::Resize(cols, rows) => {
                state.resize(cols as usize, rows as usize);
            }

            // ----------------------------------------------------------------
            // Key events
            // ----------------------------------------------------------------
            Event::Key(key) => {
                let should_quit = handle_key(state, key, &mut input_mode);
                if should_quit {
                    break;
                }
            }

            _ => continue,
        }

        render(state, &renderer, stdout)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Key handling
// ---------------------------------------------------------------------------

/// Returns `true` when the user has requested to quit.
fn handle_key(state: &mut SheetsState, key: KeyEvent, mode: &mut InputMode) -> bool {
    match *mode {
        InputMode::Search => handle_search_key(state, key, mode),
        InputMode::Pending(pending) => handle_pending_key(state, key, mode, pending),
        InputMode::Normal => handle_normal_key(state, key, mode),
    }
}

fn handle_normal_key(
    state: &mut SheetsState,
    key: KeyEvent,
    mode: &mut InputMode,
) -> bool {
    let no_mod = key.modifiers == KeyModifiers::NONE;
    let shift = key.modifiers == KeyModifiers::SHIFT;
    let ctrl = key.modifiers == KeyModifiers::CONTROL;

    match key.code {
        // ---- quit ----
        KeyCode::Char('q') if no_mod => return true,
        KeyCode::Char('c') if ctrl => return true,

        // ---- arrow navigation ----
        KeyCode::Down => state.select_down(),
        KeyCode::Up => state.select_up(),
        KeyCode::Left => state.select_left(),
        KeyCode::Right => state.select_right(),

        // ---- vim hjkl ----
        KeyCode::Char('j') if no_mod => state.select_down(),
        KeyCode::Char('k') if no_mod => state.select_up(),
        KeyCode::Char('h') if no_mod => state.select_left(),
        KeyCode::Char('l') if no_mod => state.select_right(),

        // ---- page navigation ----
        KeyCode::PageDown => state.page_down(),
        KeyCode::PageUp => state.page_up(),
        KeyCode::Char('d') if ctrl => state.half_page_down(),
        KeyCode::Char('u') if ctrl => state.half_page_up(),

        // ---- row jumps ----
        KeyCode::Home => state.go_to_top(),
        KeyCode::End => state.go_to_bottom(),
        // gg — first key: arm the pending state
        KeyCode::Char('g') if no_mod => {
            *mode = InputMode::Pending(PendingKey::LowercaseG);
            return false;
        }
        // G — go to bottom
        KeyCode::Char('G') if shift | no_mod => state.go_to_bottom(),

        // ---- vim screen-line jumps (H / M / L) ----
        KeyCode::Char('H') if shift | no_mod => state.go_to_top_visible(),
        KeyCode::Char('M') if shift | no_mod => state.go_to_middle_visible(),
        KeyCode::Char('L') if shift | no_mod => state.go_to_bottom_visible(),

        // ---- column jumps ----
        KeyCode::Char('0') if no_mod => state.go_to_first_col(),
        KeyCode::Char('$') if no_mod => state.go_to_last_col(),

        // ---- search ----
        KeyCode::Char('/') if no_mod => {
            *mode = InputMode::Search;
            state.begin_search(SearchDirection::Forward);
        }
        KeyCode::Char('?') if no_mod => {
            *mode = InputMode::Search;
            state.begin_search(SearchDirection::Backward);
        }
        KeyCode::Char('n') if no_mod => {
            state.search_next();
        }
        KeyCode::Char('N') if shift | no_mod => {
            state.search_prev();
        }

        KeyCode::Esc => { /* no-op in normal mode */ }

        _ => {}
    }

    false
}

fn handle_search_key(
    state: &mut SheetsState,
    key: KeyEvent,
    mode: &mut InputMode,
) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.search_cancel();
            *mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            state.search_commit();
            *mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            state.search_backspace();
        }
        KeyCode::Char(ch)
            if key.modifiers == KeyModifiers::NONE
                || key.modifiers == KeyModifiers::SHIFT =>
        {
            state.search_append(ch);
        }
        _ => {}
    }
    false
}

fn handle_pending_key(
    state: &mut SheetsState,
    key: KeyEvent,
    mode: &mut InputMode,
    pending: PendingKey,
) -> bool {
    // Always clear the pending state first; the match below re-arms if needed.
    *mode = InputMode::Normal;

    match pending {
        PendingKey::LowercaseG => {
            if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::NONE {
                state.go_to_top();
            }
            // Any other key: pending consumed, no action.
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(
    state: &SheetsState,
    renderer: &UiRenderer,
    stdout: &mut impl Write,
) -> anyhow::Result<()> {
    let frame = renderer
        .draw_ui(state)
        .unwrap_or_else(|e| format!("render error: {e}"));

    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All),
        Print(&frame),
    )?;
    stdout.flush()?;
    Ok(())
}
