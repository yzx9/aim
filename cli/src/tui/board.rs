// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;

use aimcal_core::{Id, LooseDateTime, Priority, Todo, TodoPatch, TodoStatus};
use colored::Color as ColoredColor;
use jiff::Zoned;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::todo_formatter::get_color_due_impl;
use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::Dispatcher;
use crate::util::format_datetime;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Action emitted by the board TUI when it exits for async processing.
#[derive(Debug, Clone)]
pub enum BoardAction {
    /// Move a card to a target column status.
    Move {
        card_id: Id,
        target_status: TodoStatus,
    },
    /// User requested quit.
    Quit,
    /// User requested refresh.
    Refresh,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardMode {
    Navigate,
    MoveTarget,
}

/// Column definition: status variant + display name.
pub(crate) const COLUMN_DEFS: [(TodoStatus, &str); 4] = [
    (TodoStatus::NeedsAction, "Backlog"),
    (TodoStatus::InProcess, "In Progress"),
    (TodoStatus::Completed, "Done"),
    (TodoStatus::Cancelled, "Cancelled"),
];

/// Owned snapshot of a `Todo` for rendering (avoids repeated trait calls).
#[derive(Debug, Clone)]
pub(crate) struct CardData {
    uid: String,
    short_id: Option<String>,
    summary: String,
    due: Option<LooseDateTime>,
    due_display: String,
    priority: Priority,
    status: TodoStatus,
    percent_complete: Option<u8>,
}

/// A single Kanban column with its cards and scroll offset.
#[derive(Debug, Clone)]
struct Column {
    status: TodoStatus,
    name: &'static str,
    cards: Vec<CardData>,
    scroll_offset: u16,
}

/// Saved cursor state for persistence across reloads.
#[derive(Debug, Clone)]
pub struct CursorSnapshot {
    selected_col: usize,
    selected_card: usize,
    scroll_offsets: Vec<u16>,
}

/// All board state passed through `RefCell<BoardState>` to the `Component`.
#[derive(Debug, Clone)]
pub struct BoardState {
    columns: Vec<Column>,
    selected_col: usize,
    selected_card: usize,
    mode: BoardMode,
    move_source_col: usize,
    move_source_card: usize,
    today_filter: bool,
    pub pending_action: Option<BoardAction>,
    pub error_message: Option<String>,
    pub error_timestamp: Option<std::time::Instant>,
    now: Zoned,
    truncation_warning: bool,
}

impl BoardState {
    /// Build board state from per-status card groups.
    pub fn new(
        groups: &[(TodoStatus, Vec<CardData>)],
        now: Zoned,
        truncation_warning: bool,
    ) -> Self {
        let columns: Vec<Column> = COLUMN_DEFS
            .iter()
            .map(|&(status, name)| {
                let mut cards = groups
                    .iter()
                    .find(|(s, _)| *s == status)
                    .map(|(_, cards)| cards.clone())
                    .unwrap_or_default();
                // Sort: due date ascending, no-due at bottom
                cards.sort_by(|a, b| compare_due(a.due.as_ref(), b.due.as_ref()));
                Column {
                    status,
                    name,
                    cards,
                    scroll_offset: 0,
                }
            })
            .collect();

        Self {
            columns,
            selected_col: 0,
            selected_card: 0,
            mode: BoardMode::Navigate,
            move_source_col: 0,
            move_source_card: 0,
            today_filter: false,
            pending_action: None,
            error_message: None,
            error_timestamp: None,
            now,
            truncation_warning,
        }
    }

    /// Save cursor position for restoration after reload.
    pub fn save_cursor(&self) -> CursorSnapshot {
        CursorSnapshot {
            selected_col: self.selected_col,
            selected_card: self.selected_card,
            scroll_offsets: self.columns.iter().map(|c| c.scroll_offset).collect(),
        }
    }

    /// Restore cursor from a snapshot, clamping to valid ranges.
    pub fn restore_cursor(&mut self, snap: CursorSnapshot) {
        self.selected_col = snap.selected_col.min(self.columns.len().saturating_sub(1));
        let cards_len = self.columns[self.selected_col].cards.len();
        self.selected_card = snap.selected_card.min(cards_len.saturating_sub(1));
        for (i, offset) in snap.scroll_offsets.into_iter().enumerate() {
            if let Some(col) = self.columns.get_mut(i) {
                let total: u16 = col.cards.iter().map(card_height).sum();
                col.scroll_offset = offset.min(total);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Card data helpers
// ---------------------------------------------------------------------------

/// Build a `CardData` from any `impl Todo`.
pub fn card_data_from_todo(todo: &impl Todo) -> CardData {
    let due = todo.due();
    let due_display = due
        .as_ref()
        .map(|d| format_datetime(d.clone()))
        .unwrap_or_default();
    let short_id = todo.short_id().map(|n| n.to_string());
    CardData {
        uid: todo.uid().into_owned(),
        short_id,
        summary: todo.summary().replace('\n', " "),
        due,
        due_display,
        priority: todo.priority(),
        status: todo.status(),
        percent_complete: todo.percent_complete(),
    }
}

/// Build an `Id` from card data (prefers short ID for usability).
fn make_id(card: &CardData) -> Id {
    if let Some(ref sid) = card.short_id {
        Id::ShortIdOrUid(sid.clone())
    } else {
        Id::Uid(card.uid.clone())
    }
}

// ---------------------------------------------------------------------------
// Sort / normalization helpers
// ---------------------------------------------------------------------------

fn compare_due(a: Option<&LooseDateTime>, b: Option<&LooseDateTime>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(da), Some(db)) => da.date().cmp(&db.date()),
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// Build a `TodoPatch` for moving a card to a target status with normalized percent.
pub fn build_move_patch(target_status: TodoStatus) -> TodoPatch {
    let percent_complete = match target_status {
        TodoStatus::Completed => Some(Some(100)),
        TodoStatus::NeedsAction | TodoStatus::InProcess => Some(None),
        TodoStatus::Cancelled => None,
    };
    TodoPatch {
        status: Some(target_status),
        percent_complete,
        ..TodoPatch::default()
    }
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

fn colored_to_ratatui(color: ColoredColor) -> Color {
    match color {
        ColoredColor::Yellow => Color::Yellow,
        ColoredColor::Red => Color::Red,
        ColoredColor::TrueColor { r, g, b } => Color::Rgb(r, g, b),
        _ => Color::Reset,
    }
}

fn get_due_color(due: &LooseDateTime, now: &Zoned) -> Option<Color> {
    get_color_due_impl(due, now).map(colored_to_ratatui)
}

fn format_priority_badge(priority: Priority) -> &'static str {
    match priority {
        Priority::P1 | Priority::P2 | Priority::P3 => "!!!",
        Priority::P4 | Priority::P5 | Priority::P6 => "!!",
        Priority::P7 | Priority::P8 | Priority::P9 => "!",
        Priority::None => "",
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Card height: 2 borders + 2 content lines + 1 for progress bar if `percent_complete` is set.
fn card_height(card: &CardData) -> u16 {
    if card.percent_complete.is_some() {
        5
    } else {
        4
    }
}

fn truncate_unicode(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        return s.to_string();
    }
    let mut result = String::new();
    let mut current_width = 0;
    for grapheme in s.graphemes(true) {
        let gw = UnicodeWidthStr::width(grapheme);
        if current_width + gw + 3 > max_width {
            break;
        }
        result.push_str(grapheme);
        current_width += gw;
    }
    result.push_str("...");
    result
}

fn render_card(
    card: &CardData,
    is_selected: bool,
    is_moving: bool,
    dimmed: bool,
    area: Rect,
    buf: &mut Buffer,
    now: &Zoned,
) {
    let border_color = if is_moving {
        Color::Yellow
    } else if is_selected {
        Color::Blue
    } else {
        Color::Reset
    };

    let border_type = if is_selected || is_moving {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    let text_style = if dimmed {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    // Line 1: "#ID summary..."
    let id_str = card.short_id.as_deref().unwrap_or(&card.uid);
    let available = area.width.saturating_sub(4) as usize; // borders + padding
    let id_display = format!("#{id_str} ");
    let id_display_width = UnicodeWidthStr::width(id_display.as_str());
    let summary_display = if card.summary.is_empty() {
        "(no title)".to_string()
    } else {
        truncate_unicode(&card.summary, available.saturating_sub(id_display_width))
    };
    let summary_style = if card.summary.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        text_style
    };
    let line1 = Line::from(vec![
        Span::styled(id_display, Style::default().fg(Color::Cyan)),
        Span::styled(summary_display, summary_style),
    ]);

    // Line 2: "priority due_date [progress%]"
    let mut spans: Vec<Span> = vec![];
    let badge = format_priority_badge(card.priority);
    if !badge.is_empty() {
        spans.push(Span::styled(
            format!("{badge} "),
            Style::default().fg(Color::Red),
        ));
    }
    if !card.due_display.is_empty() {
        let due_color = card
            .due
            .as_ref()
            .and_then(|d| get_due_color(d, now))
            .map_or(text_style, |c| text_style.fg(c));
        spans.push(Span::styled(&card.due_display, due_color));
    }
    if let Some(pct) = card.percent_complete {
        spans.push(Span::styled(format!(" [{pct}%]"), text_style));
    }
    let line2 = Line::from(spans);

    // Block with border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    block.render(area, buf);

    // Render text lines
    let card_lines = vec![line1, line2];
    Paragraph::new(card_lines)
        .style(text_style)
        .render(inner, buf);

    // Progress bar (2 extra lines) if percent_complete is set
    if let Some(pct) = card.percent_complete {
        render_progress_bar(pct, inner, buf, dimmed);
    }
}

fn render_progress_bar(percent: u8, area: Rect, buf: &mut Buffer, dimmed: bool) {
    let pct = u16::from(percent.min(100));
    let width = area.width.saturating_sub(2);
    if width == 0 || area.height < 3 {
        return;
    }
    let filled = (width * pct) / 100;
    let empty = width - filled;

    let bar_style = if dimmed {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green)
    };

    let bar_str = format!(
        "{}{}",
        "=".repeat(filled as usize),
        "-".repeat(empty as usize)
    );
    let bar_line = Line::from(Span::styled(bar_str, bar_style));
    buf.set_line(area.x + 1, area.y + 2, &bar_line, width);
}

fn render_column(column: &Column, state: &BoardState, area: Rect, buf: &mut Buffer) {
    let col_idx = state
        .columns
        .iter()
        .position(|c| c.status == column.status)
        .unwrap_or(0);

    // Column header with count
    let header = Line::from(Span::styled(
        format!(" {} ({}) ", column.name, column.cards.len()),
        Style::default().add_modifier(Modifier::BOLD),
    ));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .title(header.centered());
    let inner = block.inner(area);
    block.render(area, buf);

    if column.cards.is_empty() {
        let empty_text = Paragraph::new("(empty)")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        let centered_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1) / 2,
            width: inner.width,
            height: 1,
        };
        empty_text.render(centered_area, buf);
        return;
    }

    // Render visible cards with scroll
    let visible_height = inner.height;
    let mut y: u16 = 0;

    for (i, card) in column.cards.iter().enumerate() {
        let h = card_height(card);
        if y < column.scroll_offset {
            y += h;
            continue;
        }
        let card_y = inner.y + y - column.scroll_offset;
        if card_y >= inner.y + visible_height {
            break;
        }
        let card_height_clamped = h.min(inner.y + visible_height - card_y);
        let card_area = Rect {
            x: inner.x,
            y: card_y,
            width: inner.width,
            height: card_height_clamped,
        };

        let is_selected = col_idx == state.selected_col && i == state.selected_card;
        let is_moving = state.mode == BoardMode::MoveTarget
            && col_idx == state.move_source_col
            && i == state.move_source_card;

        let is_done_or_cancelled =
            matches!(card.status, TodoStatus::Completed | TodoStatus::Cancelled);
        let is_today = card
            .due
            .as_ref()
            .is_some_and(|d| d.date() == state.now.date());
        let dimmed = (state.today_filter && !is_today) || is_done_or_cancelled;

        render_card(
            card,
            is_selected,
            is_moving,
            dimmed,
            card_area,
            buf,
            &state.now,
        );
        y += h;
    }

    // Scroll indicator
    let total_height: u16 = column.cards.iter().map(card_height).sum();
    if total_height > visible_height {
        let indicator = if column.scroll_offset > 0
            && column.scroll_offset + visible_height < total_height
        {
            " ↕ "
        } else if column.scroll_offset > 0 && column.scroll_offset + visible_height >= total_height
        {
            " ↑ "
        } else {
            " ↓ "
        };
        let scroll_span = Span::styled(indicator, Style::default().fg(Color::DarkGray));
        buf.set_span(
            inner.x + inner.width.saturating_sub(3),
            inner.y,
            &scroll_span,
            3,
        );
    }
}

fn render_footer(state: &BoardState, area: Rect, buf: &mut Buffer) {
    let mut spans: Vec<Span> = vec![];

    match state.mode {
        BoardMode::Navigate => {
            spans.push(Span::styled(
                " NAVIGATE ",
                Style::default().fg(Color::Black).bg(Color::Blue),
            ));
            spans.push(Span::raw("  "));
            spans.push(Span::styled("h/l", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" columns  "));
            spans.push(Span::styled("j/k", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" cards  "));
            spans.push(Span::styled("m", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" move  "));
            spans.push(Span::styled("t", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" today  "));
            spans.push(Span::styled("r", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" refresh  "));
            spans.push(Span::styled("q", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" quit"));
        }
        BoardMode::MoveTarget => {
            spans.push(Span::styled(
                " MOVE ",
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ));
            spans.push(Span::raw("  "));
            spans.push(Span::styled("h/l", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" target column  "));
            spans.push(Span::styled("Enter", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" commit  "));
            spans.push(Span::styled("Esc", Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" cancel"));
        }
    }

    if state.today_filter {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            " TODAY ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
    }

    if let Some(ref err) = state.error_message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Error: {err}"),
            Style::default().fg(Color::Red),
        ));
    }

    if state.truncation_warning {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "TRUNCATED",
            Style::default().fg(Color::Yellow),
        ));
    }

    let line = Line::from(spans);
    buf.set_line(area.x, area.y, &line, area.width);
}

// ---------------------------------------------------------------------------
// Board component
// ---------------------------------------------------------------------------

/// Zero-sized Kanban board component.
pub struct Board;

impl Board {
    pub fn new() -> Self {
        Board
    }
}

impl Component<BoardState> for Board {
    fn render(&self, store: &RefCell<BoardState>, area: Rect, buf: &mut Buffer) {
        let state = store.borrow();

        // Terminal width check
        if area.width < 120 {
            let warning = Paragraph::new("Terminal too narrow. Minimum 120 columns required.")
                .style(Style::default().fg(Color::Yellow))
                .centered();
            let centered = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1) / 2,
                width: area.width,
                height: 1,
            };
            warning.render(centered, buf);
            return;
        }

        // Layout: columns (fill) + footer (1 line)
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
        let areas = layout.split(area);
        let columns_area = areas[0];
        let footer_area = areas[1];

        // Split into 4 equal columns
        let col_layout = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ]);
        let col_areas = col_layout.split(columns_area);

        for (i, column) in state.columns.iter().enumerate() {
            if let Some(col_area) = col_areas.get(i) {
                render_column(column, &state, *col_area, buf);
            }
        }

        render_footer(&state, footer_area, buf);
    }

    fn get_cursor_position(&self, _store: &RefCell<BoardState>, _area: Rect) -> Option<(u16, u16)> {
        None
    }

    fn on_key(
        &mut self,
        _dispatcher: &mut Dispatcher,
        store: &RefCell<BoardState>,
        _area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        let mut state = store.borrow_mut();

        // Global: Ctrl+C always quits
        if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
            state.pending_action = Some(BoardAction::Quit);
            return Some(Message::Exit);
        }

        match state.mode {
            BoardMode::Navigate => handle_navigate(&mut state, event),
            BoardMode::MoveTarget => handle_move_target(&mut state, event),
        }
    }
}

// ---------------------------------------------------------------------------
// Keyboard handling
// ---------------------------------------------------------------------------

fn handle_navigate(state: &mut BoardState, event: KeyEvent) -> Option<Message> {
    match event.code {
        KeyCode::Char('q') => {
            state.pending_action = Some(BoardAction::Quit);
            Some(Message::Exit)
        }
        KeyCode::Char('r') => {
            state.pending_action = Some(BoardAction::Refresh);
            Some(Message::Exit)
        }
        KeyCode::Char('t') => {
            state.today_filter = !state.today_filter;
            Some(Message::Handled)
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if state.selected_col > 0 {
                state.selected_col -= 1;
                state.selected_card = 0;
                state.columns[state.selected_col].scroll_offset = 0;
            }
            Some(Message::Handled)
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if state.selected_col < state.columns.len() - 1 {
                state.selected_col += 1;
                state.selected_card = 0;
                state.columns[state.selected_col].scroll_offset = 0;
            }
            Some(Message::Handled)
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let cards_len = state.columns[state.selected_col].cards.len();
            if cards_len > 0 && state.selected_card < cards_len - 1 {
                state.selected_card += 1;
                ensure_card_visible(state);
            }
            Some(Message::Handled)
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.selected_card > 0 {
                state.selected_card -= 1;
                ensure_card_visible(state);
            }
            Some(Message::Handled)
        }
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_half_page_down(state);
            Some(Message::Handled)
        }
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_half_page_up(state);
            Some(Message::Handled)
        }
        KeyCode::Char('m') => {
            if state.columns[state.selected_col].cards.is_empty() {
                None
            } else {
                state.mode = BoardMode::MoveTarget;
                state.move_source_col = state.selected_col;
                state.move_source_card = state.selected_card;
                Some(Message::Handled)
            }
        }
        KeyCode::Esc => {
            state.error_message = None;
            state.error_timestamp = None;
            Some(Message::Handled)
        }
        _ => None,
    }
}

fn handle_move_target(state: &mut BoardState, event: KeyEvent) -> Option<Message> {
    match event.code {
        KeyCode::Esc => {
            state.mode = BoardMode::Navigate;
            state.selected_col = state.move_source_col;
            state.selected_card = state.move_source_card;
            Some(Message::Handled)
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if state.selected_col > 0 {
                state.selected_col -= 1;
                state.selected_card = 0;
            }
            Some(Message::Handled)
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if state.selected_col < state.columns.len() - 1 {
                state.selected_col += 1;
                state.selected_card = 0;
            }
            Some(Message::Handled)
        }
        KeyCode::Enter => {
            let source_col = state.move_source_col;
            let source_card = state.move_source_card;
            let target_col = state.selected_col;

            // Same column: cancel move
            if source_col == target_col {
                state.mode = BoardMode::Navigate;
                state.selected_card = source_card;
                return Some(Message::Handled);
            }

            let target_status = state.columns[target_col].status;
            let card = &state.columns[source_col].cards[source_card];
            let card_id = make_id(card);

            state.pending_action = Some(BoardAction::Move {
                card_id,
                target_status,
            });
            Some(Message::Exit)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Scroll helpers
// ---------------------------------------------------------------------------

/// Estimated visible height per column (viewport minus borders).
const ESTIMATED_VISIBLE_HEIGHT: u16 = 30;

fn visible_height_for_col(col: &Column) -> u16 {
    // Use a reasonable estimate; actual height is determined at render time
    let _ = col;
    ESTIMATED_VISIBLE_HEIGHT
}

fn ensure_card_visible(state: &mut BoardState) {
    let col = &mut state.columns[state.selected_col];
    if col.cards.is_empty() {
        return;
    }
    let card_y: u16 = col
        .cards
        .iter()
        .take(state.selected_card)
        .map(card_height)
        .sum();
    let card_h = card_height(&col.cards[state.selected_card]);
    let visible = visible_height_for_col(col);

    if card_y < col.scroll_offset {
        col.scroll_offset = card_y;
    } else if card_y + card_h > col.scroll_offset + visible {
        col.scroll_offset = card_y + card_h.saturating_sub(visible);
    }
}

fn scroll_half_page_down(state: &mut BoardState) {
    let col = &mut state.columns[state.selected_col];
    if col.cards.is_empty() {
        return;
    }
    let half_page = visible_height_for_col(col) / 2;
    let total: u16 = col.cards.iter().map(card_height).sum();
    col.scroll_offset =
        (col.scroll_offset + half_page).min(total.saturating_sub(visible_height_for_col(col)));

    // Advance selected_card to first card at or below scroll position
    let mut y: u16 = 0;
    for (i, card) in col.cards.iter().enumerate() {
        let h = card_height(card);
        if y + h > col.scroll_offset {
            state.selected_card = i;
            break;
        }
        y += h;
    }
}

fn scroll_half_page_up(state: &mut BoardState) {
    let col = &mut state.columns[state.selected_col];
    if col.cards.is_empty() {
        return;
    }
    let half_page = visible_height_for_col(col) / 2;
    col.scroll_offset = col.scroll_offset.saturating_sub(half_page);

    // Move selected_card to first card at or below scroll position
    let mut y: u16 = 0;
    for (i, card) in col.cards.iter().enumerate() {
        let h = card_height(card);
        if y + h > col.scroll_offset {
            state.selected_card = i;
            break;
        }
        y += h;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_due_sorts_none_last() {
        let due_a = LooseDateTime::DateOnly(jiff::civil::date(2026, 4, 26));
        let due_b = LooseDateTime::DateOnly(jiff::civil::date(2026, 4, 27));

        assert_eq!(
            compare_due(Some(&due_a.clone()), None),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_due(None, Some(&due_b.clone())),
            std::cmp::Ordering::Greater
        );
        assert_eq!(compare_due(None, None), std::cmp::Ordering::Equal);
        assert_eq!(
            compare_due(Some(&due_a), Some(&due_b)),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn build_move_patch_normalizes_percent() {
        let patch = build_move_patch(TodoStatus::Completed);
        assert_eq!(patch.status, Some(TodoStatus::Completed));
        assert_eq!(patch.percent_complete, Some(Some(100)));

        let patch = build_move_patch(TodoStatus::NeedsAction);
        assert_eq!(patch.percent_complete, Some(None));

        let patch = build_move_patch(TodoStatus::InProcess);
        assert_eq!(patch.percent_complete, Some(None));

        let patch = build_move_patch(TodoStatus::Cancelled);
        assert_eq!(patch.percent_complete, None);
    }

    #[test]
    fn truncate_unicode_basic() {
        assert_eq!(truncate_unicode("hello", 10), "hello");
        assert_eq!(truncate_unicode("hello world", 8), "hello...");
    }

    #[test]
    fn priority_badge_format() {
        assert_eq!(format_priority_badge(Priority::P1), "!!!");
        assert_eq!(format_priority_badge(Priority::P5), "!!");
        assert_eq!(format_priority_badge(Priority::P9), "!");
        assert_eq!(format_priority_badge(Priority::None), "");
    }

    #[test]
    fn cursor_snapshot_clamps_on_restore() {
        let now = Zoned::now();
        let state = BoardState::new(&[], now, false);
        // columns are empty, so selected_card should be 0
        let snap = CursorSnapshot {
            selected_col: 0,
            selected_card: 5,
            scroll_offsets: vec![100, 200, 300, 400],
        };
        let mut state = state;
        state.restore_cursor(snap);
        assert_eq!(state.selected_col, 0);
        assert_eq!(state.selected_card, 0); // clamped to 0 (empty column)
    }

    #[test]
    fn card_height_with_and_without_progress() {
        let card_no_progress = CardData {
            uid: "test".to_string(),
            short_id: Some("1".to_string()),
            summary: "test".to_string(),
            due: None,
            due_display: String::new(),
            priority: Priority::None,
            status: TodoStatus::NeedsAction,
            percent_complete: None,
        };
        assert_eq!(card_height(&card_no_progress), 4);

        let card_with_progress = CardData {
            percent_complete: Some(50),
            ..card_no_progress.clone()
        };
        assert_eq!(card_height(&card_with_progress), 5);
    }
}
