# 00006: Calendar-Native Kanban Board

**Status:** Reviewed (eng review + codex outside voice)
**Date:** 2026-04-25
**Supersedes:** Office-hours design doc (yzx9-main-design-20260424-165457.md)

## Problem Statement

AIM has a table-based `aim todo list` view that works for scanning tasks, but doesn't give a spatial sense of workflow progress. A Kanban board view where todos are cards in status columns makes it immediately visible where work is stuck, what's overdue, and what needs attention.

## Design

Launch via `aim todo board` (alias: `aim board`). An interactive ratatui TUI session that:

1. Loads all todos from SQLite via `Aim::list_todos` with status-filtered queries (4 queries, one per `TodoStatus`, `Pager { limit: 500, offset: 0 }` per query), groups by `TodoStatus` into 4 columns. If any column returns 500 results, show a truncation warning.

2. Renders cards with summary, short ID, priority badge, and due date with time-pressure color coding.

3. Supports vim-style navigation (h/l between columns, j/k between cards).

4. Supports move mode (press `m` on a card, then h/l to choose target column, h/l clamp at first/last column, Enter to commit, Esc to cancel).

5. Status transitions use `Aim::update_todo` with `TodoPatch { status: Some(target_status), percent_complete: ..., .. }`. Moving to Done is identical to `aim todo done` with normalization.

6. "Today" filter (toggle with `t`): "Due today" means `due().local_date() == today` using local timezone. When active, cards not due today (including those with no due date) are rendered with `Style::default().fg(Color::DarkGray)`, footer shows `TODAY` indicator.

7. Footer shows current mode (Navigate/Move) and keybindings.

8. Empty columns display centered "(empty)" placeholder. Columns never collapse.

9. `q` quits the board. `Escape` cancels move mode.

### Column Mapping

| TodoStatus     | Column Name   |
|----------------|---------------|
| NeedsAction    | Backlog       |
| InProcess      | In Progress   |
| Completed      | Done          |
| Cancelled      | Cancelled     |

Note: Cancelled column uses its real status name, not "Archive", because moving a card there is a destructive action (marks todo as cancelled). Honest naming prevents accidental destructive actions.

### Card Rendering Spec

- Base height: 2 lines per card (line 1: `#ID summary...`, line 2: `priority due_date [progress]`). Cards with `percent_complete` gain an additional 2-line progress bar (4 lines total).
- Summary truncates with `...` at column width minus ID and padding. Use ratatui's unicode-aware APIs for truncation (never manual `&str[..n]` slicing).
- `percent_complete` of `None` is omitted from rendering entirely. When present, rendered as a 2-line progress bar after the card. Clamp to 0-100 for malformed data.
- Selected card: blue border (`ratatui::style::Color::Blue`).
- Moving card: amber border (`ratatui::style::Color::Yellow`).
- Done/Cancelled cards: dimmed text (`Style::default().fg(Color::DarkGray)`).

### Card Sort Order

Cards within each column are sorted by due date ascending. Todos without a due date appear at the bottom.

**Implementation:** Add a new `TodoSort` variant that handles no-due-at-bottom semantics at the DB level (following the existing `Priority { none_first }` pattern). If the DB cannot provide the desired order, sort client-side in Rust.

### Time-Pressure Color Mapping

Reuse the exact color logic from `cli/src/todo_formatter.rs`. The board calls `get_color_due_impl` and converts `colored::Color` to `ratatui::style::Color` via a conversion function.

Actual RGB values from `todo_formatter.rs:154-160`:

| Tier | RGB | Color type |
|------|-----|------------|
| Overdue < 24h | `TrueColor { r: 255, g: 162, b: 162 }` | Lightest red |
| Overdue < 48h | `TrueColor { r: 251, g: 43, b: 55 }` | Medium red |
| Overdue < 72h | `TrueColor { r: 193, g: 2, b: 7 }` | Dark red |
| Overdue >= 72h | `TrueColor { r: 130, g: 24, b: 26 }` | Darkest red |
| Due today (not overdue) | `Yellow` | — |
| Future / no due date | default foreground | — |
| Priority badge | `Red` | — |

### Status Transition Normalization

When moving cards between columns, normalize `percent_complete`:

| Target status | percent_complete |
|---------------|------------------|
| Completed | `Some(100)` |
| NeedsAction | `None` |
| InProcess | `None` |
| Cancelled | leave as-is |

This ensures data consistency (no Completed todos at 20%, no InProcess todos at 100%).

### Column Scrolling

- Each column has its own scroll offset.
- `j`/`k` scroll one item.
- `Ctrl+d`/`Ctrl+u` scroll half-page.
- Scroll indicator renders at column edge when items exceed viewport.

### TUI Architecture

The board implements the existing `Component<BoardState>` trait from `cli/src/tui/component.rs`, using the `Dispatcher` and `App` framework. The board's state management follows the same pattern as `TodoStore`/`EventStore`:

- `BoardState` holds all board data (columns, cards, cursor, scroll offsets, mode).
- `BoardState` has a `pending_action: Option<BoardAction>` field.
- When a move is committed, the Component sets `pending_action` and returns `Message::Exit`.
- The outer function reads `pending_action` from the returned store.
- Outer loop: match on `BoardAction`, perform async work (`update_todo`), reload, rebuild `BoardState`, call `run()` again.

```rust
pub enum BoardAction {
    Move { card_short_id: Id, target_status: TodoStatus },
    Quit,
    Refresh,
    Edit { card_short_id: Id },
}
```

### State Persistence Across Reloads

After each move (which triggers a BoardState rebuild), persist and restore:
- Selected column index
- Selected card index within column
- Per-column scroll offsets

This prevents the cursor from bouncing to the top after every card move.

### Event Loop: Poll-Based Tick

Use `crossterm::event::poll()` with a ~100ms timeout instead of blocking `event::read()`. This enables:
- Timed error footer display (3-second auto-dismiss via timestamp check)
- Terminal resize detection
- Future auto-refresh capability

### Error Handling

When a card move fails (CalDAV conflict, stale short ID): show inline error message in footer, track error timestamp in `BoardState`, auto-clear after 3 seconds on next tick. Revert board to pre-move state.

### Terminal Width

- Minimum: 120 columns. Check at startup.
- Dynamic resize: on each tick, check terminal area. If below 120 columns, show centered warning message instead of the board.
- 4 columns at 120 chars gives ~28 chars per column after borders.

### Async Strategy

The existing TUI uses `ratatui::run()` with a synchronous closure, but `Aim::update_todo` and `Aim::list_todos` are async. Strategy:

1. Load all todos into `BoardState` before entering the TUI event loop.
2. The event loop returns `Message::Exit` when an action is pending.
3. Outer loop: read `pending_action`, perform async work (update_todo, reload), rebuild `BoardState` preserving cursor state, re-enter event loop.

### CLI Registration

Follow the `done` pattern from `cli/src/cli.rs`:
- Register `CmdTodoBoard::command()` under the `todo` subcommand group
- Register `CmdTodoBoard::command()` at the root level for the `aim board` alias
- Add `Board(CmdTodoBoard)` variant to `Commands` enum
- Add match arms in both `Cli::from()` and `Commands::run()`

## Constraints

- Terminal UI only (ratatui). No mouse drag-and-drop.
- Must reuse existing todo data model — no new database tables or fields.
- Must preserve CalDAV/iCalendar sync semantics when changing status.
- Keyboard-driven interaction (vim-style hjkl navigation).
- Board merges all calendars (no `--calendar` filter in v1).

## Deferred to v2

(to be written to `docs/specs/00007-board-v2-features.md` during implementation)

- Focus mode (only show In Progress column + related context)
- Calendar filtering (`--calendar` flag)
- `percent_complete` auto-update on card movement
- Auto-refresh when CalDAV sync pulls changes
- Inline card editing
- Board goes stale during long sessions (manual refresh `r` in v1)

## Success Criteria

- `aim todo board` launches and renders all todos in 4 columns within 500ms.
- Card movement between columns persists to SQLite and is reflected in `aim todo list`.
- Due date color coding reuses the existing overdue gradient from `todo_formatter.rs`.
- Board handles 100+ todos without layout issues (per-column scrolling).
- Board renders correctly with zero todos (all columns show "(empty)").
- Minimum terminal width of 120 columns. Warns and exits if narrower.
- Dynamic resize: layout adjusts when terminal is resized during session.
- State persistence: cursor position survives card moves without bouncing to top.

## Files Touched

- **New:** `cli/src/tui/board.rs` — Board component, state, rendering, keyboard handling
- **Modified:** `cli/src/tui.rs` — Register board module, add `run_board_editor`
- **Modified:** `cli/src/cmd_todo.rs` — Add `CmdTodoBoard` command
- **Modified:** `cli/src/cli.rs` — Register subcommand + top-level alias
- **New:** Tests for BoardState logic, clap parsing, status transitions

## Review History

- Eng review (2026-04-25): 8 issues found and resolved
  - Color values corrected from design doc
  - Poll-based tick loop for error timer + resize
  - Top-level alias registration pattern
  - DRY color mapping via conversion function
  - Dynamic terminal resize handling
  - State persistence across reloads
  - Client-side sort for no-due-at-bottom
  - Status/percent_complete normalization
- Codex outside voice (2026-04-25): 5 actionable findings
  - State reset after move (resolved: persist cursor)
  - Sort order incorrect for empty due dates (resolved: new DB sort variant)
  - "Archive" naming hides destructive action (resolved: renamed to "Cancelled")
  - Status-only moves leave inconsistent state (resolved: normalize percent_complete)
  - Architecture: reuse existing Component/Dispatcher (resolved: user chose reuse)
