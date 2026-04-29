# 00007: Board v2 Features

**Status:** Reviewed (eng review)
**Date:** 2026-04-28
**Depends on:** 00006 (Kanban Board v1)

## Problem Statement

The v1 kanban board provides basic card visualization and column-to-column movement, but it is read-only. Users must leave the board to view card details, edit fields, or search for a specific card. The board also has no auto-refresh, making it stale during long sessions (e.g., when CalDAV sync pulls changes in the background).

The v2 features aim to make the board a primary interface rather than a read-only overview: reduce context switching, enable inline inspection and editing, and keep the board current without manual intervention.

## Features

### F1: Card Detail View

Press `Enter` on a selected card to open the full todo editor, then return to the board with cursor preserved.

**BoardAction extension** — add `Edit { card_id: Id }` variant:

```rust
pub enum BoardAction {
    Move { card_id: Id, target_status: TodoStatus },
    Edit { card_id: Id },
    Quit,
    Refresh,
}
```

**Outer loop** (`cmd_todo.rs`, `CmdTodoBoard::run`): add `BoardAction::Edit` arm:

```rust
BoardAction::Edit { ref card_id } => {
    cursor_snap = Some(returned_state.save_cursor());
    match aim.get_todo(card_id).await {
        Ok(todo) => {
            match tui::patch_todo(aim, &todo, TodoPatch::default()) {
                Ok(Some(patch)) => {
                    match aim.update_todo(card_id, patch).await {
                        Ok(_) => state = load_board_state(aim).await?,
                        Err(e) => {
                            // update_todo failed after user edited — worst case
                            state = load_board_state(aim).await?;
                            state.error_message = Some(format!("{e}"));
                            state.error_timestamp = Some(Instant::now());
                        }
                    }
                }
                Ok(None) => state = load_board_state(aim).await?, // user cancelled
                Err(e) => {
                    state = load_board_state(aim).await?;
                    state.error_message = Some(format!("{e}"));
                    state.error_timestamp = Some(Instant::now());
                }
            }
        }
        Err(e) => {
            // get_todo failed (card deleted) — keep old state
            state = pre_edit_state;
            state.error_message = Some(format!("{e}"));
            state.error_timestamp = Some(Instant::now());
            state.pending_action = None;
        }
    }
}
```

Note: `tui::patch_todo()` returns `Option<TodoPatch>`. The caller MUST call `aim.update_todo()` to persist the edit. This follows the same pattern as `CmdTodoEdit::run` (`cmd_todo.rs:238-249`).

**Error handling** covers two failure paths:

1. `get_todo` fails (card deleted): keep old board state, show error
2. `update_todo` fails (after user edited): reload board to show current state, show error. User's edits are lost in this case — unavoidable without a command stack

**Key handling** (`board.rs`, `handle_navigate`): add `KeyCode::Enter` handler that sets `pending_action = Some(BoardAction::Edit { card_id })` and returns `Message::Exit`. Only active when the current column has cards.

**Footer**: add `Enter` to the Navigate mode keybinding display.

### F2: Search/Filter

Press `/` to enter search mode. Type a query; non-matching cards are dimmed. `Enter` accepts the filter, `Escape` clears it.

**New mode**:

```rust
enum BoardMode {
    Navigate,
    MoveTarget,
    SearchInput,
}
```

**New BoardState field**:

```rust
search_query: Option<String>,
```

Note: no `search_cursor` field needed. Cursor is always at `query.len()` since the input is append-only (no arrow keys, no Home/End).

**Filtering**: `card_matches_search(card: &CardData, query: &str) -> bool` performs case-insensitive substring match on `summary`, `description`, and `short_id`. During render, non-matching cards are dimmed using the same `Color::DarkGray` treatment as the today filter.

**Dimming helper**: with 3+ dimming conditions (today filter, search filter, done/cancelled status), extract into a helper function `should_dim_card(card, state) -> bool` rather than inline logic.

**CardData extension**: add `description: Option<String>` to `CardData`, populated from `todo.description()` in `card_data_from_todo`. Needed so search matches description content.

**Rendering**: during `SearchInput` mode, the search input **replaces the footer** (same 1-line slot). After pressing Enter to accept the filter, the normal footer returns with a `SEARCH: <query>` indicator badge (same position as the `TODAY` badge). No layout change needed, columns keep full height.

**Cursor position**: `Board::get_cursor_position` must return `Some(position)` during `SearchInput` mode. Compute position from the footer area y-coordinate + search query character offset. This requires `run_board` in `tui.rs` to handle cursor positioning (currently it ignores cursor).

**Key handling**:

- `/` in Navigate → enter SearchInput, initialize `search_query = Some(String::new())`
- Typing in SearchInput → append to query, filter in real-time
- `Backspace` → remove last character (no-op if empty)
- `Enter` → accept filter, return to Navigate (keep query active)
- `Escape` → clear query, return to Navigate

### F3: Auto-Refresh

Reload board data from the database every 30 seconds, using the existing poll-based event loop.

**State addition** (`BoardState`):

```rust
last_refresh: std::time::Instant,
```

Initialize to `Instant::now()` in `BoardState::new`.

**Tick method** — consolidate all timer-based logic into `BoardState::tick()`:

```rust
impl BoardState {
    pub fn tick(&mut self) -> Option<BoardAction> {
        // Auto-clear error after 3s (moved from run_board)
        if let Some(ts) = self.error_timestamp
            && ts.elapsed() > std::time::Duration::from_secs(3)
        {
            self.error_message = None;
            self.error_timestamp = None;
        }

        // Auto-refresh after 30s in Navigate mode only
        if self.mode == BoardMode::Navigate
            && self.last_refresh.elapsed() > std::time::Duration::from_secs(30)
        {
            self.pending_action = Some(BoardAction::Refresh);
            return Some(BoardAction::Refresh);
        }

        None
    }
}
```

**Poll loop change** (`tui.rs`, `run_board`, timeout branch): replace the existing inline error-clearing code with a call to `state.tick()`:

```rust
Ok(false) => {
    let mut state = store.borrow_mut();
    if state.tick().is_some() {
        break Ok(());
    }
}
```

This removes all direct `BoardState` field access from `tui.rs`. The `BoardMode` enum stays private.

**Important**: auto-refresh only fires in `Navigate` mode. `MoveTarget` and `SearchInput` modes skip refresh to avoid interrupting the user.

**Error handling**: if `load_board_state` fails during auto-refresh, keep the old board state, set `error_message` + `error_timestamp`, reset `last_refresh` to `Instant::now()` (prevents retry storm — without this, the stale timestamp triggers another refresh on the next tick), re-enter the board. Do not crash on background refresh failure.

**Footer hint**: change `r` label from "refresh" to "refresh now" to clarify that refresh also happens automatically.

### F4: WIP Limits & Column Stats

Visual-only WIP warnings in column headers when card count exceeds a threshold.

**Constants**:

```rust
const WIP_LIMITS: [(TodoStatus, usize); 2] = [
    (TodoStatus::NeedsAction, 15),
    (TodoStatus::InProcess, 5),
];
```

**Rendering change** (`render_column`): when `column.cards.len()` exceeds the WIP limit for that status, render the count in `Color::Red` instead of default. Format: `Backlog (18/15)` where `/15` is the limit.

Only `NeedsAction` and `InProcess` have WIP limits — `Done` and `Cancelled` are sinks, not workflow bottlenecks.

## State Persistence

**CursorSnapshot** (`board.rs`) extended to survive all state rebuilds:

```rust
pub struct CursorSnapshot {
    selected_col: usize,
    selected_card: usize,
    scroll_offsets: Vec<u16>,
    search_query: Option<String>,   // NEW: survives reloads
    today_filter: bool,              // NEW: survives reloads
}
```

`save_cursor()` captures `search_query` and `today_filter`. `restore_cursor()` restores them. This ensures auto-refresh, card edits, and card moves don't lose active filters.

## Constraints

- Terminal UI only (ratatui). No mouse support.
- Reuse existing data model — no new database tables or fields.
- Preserve CalDAV/iCalendar sync semantics (F1 edits go through `Aim::update_todo`).
- Keyboard-driven interaction (vim-style + `/` for search).
- Auto-refresh must not interrupt active moves or search input.
- All async failures handled gracefully (error message + continue).

## Deferred to v3

- **Focus mode** — show only In Progress column + context. Requires new layout mode.
- **percent_complete auto-update** — `build_move_patch` already normalizes on Completed/NeedsAction/InProcess. The remaining case (auto-increment on InProcess entry) needs a UX decision on what value to set.
- **Card reordering within column** — requires persisting user-defined sort order, which the current model (sorted by due date) does not support. Needs design decision on priority-as-proxy vs new field.
- **Multi-select** — complex interaction for a single-cursor TUI model.
- **Undo** — requires action history stack. Current architecture rebuilds state from DB on each action, making rollback easy but undo requires tracking.
- **Board configuration** — requires config file format. Low urgency since 4 columns map directly to 4 `TodoStatus` variants.
- **Calendar filtering** — `--calendar` flag on `aim board` is a simple CLI arg addition, no TUI changes needed.

## Success Criteria

- **F1**: Press Enter on a card → editor opens pre-populated → edit summary → submit → board reloads showing the change, cursor on the same card. If card was deleted, error shown in footer, board stays open.
- **F2**: Press `/` → type query → non-matching cards dim in real-time → Enter accepts → Escape clears → footer shows active search indicator. Query matches summary, description, and short_id. Search survives board reloads.
- **F3**: Board reloads automatically every 30s without user action. Cursor position preserved. Does not interrupt MoveTarget or SearchInput mode. Refresh failures show error, don't crash.
- **F4**: Column headers show count. Count turns red when exceeding WIP limit. Only applies to Backlog and In Progress.

## Files Touched

| File                   | Change                                                                                                                                                                                                                                                                                                            |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cli/src/tui/board.rs` | `BoardAction::Edit` variant, `BoardMode::SearchInput`, `search_query` + `description` fields on `BoardState`/`CardData`, `last_refresh` field, `tick()` method, `WIP_LIMITS` constant, `should_dim_card` helper, `CursorSnapshot` extended, updated rendering and key handling for all 4 features, ~35 unit tests |
| `cli/src/cmd_todo.rs`  | `BoardAction::Edit` match arm in `CmdTodoBoard::run` with error handling                                                                                                                                                                                                                                          |
| `cli/src/tui.rs`       | Replace inline error-clearing with `state.tick()` call, add cursor position handling for search input                                                                                                                                                                                                             |
