// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod app;
// Board state invariants guarantee valid column/card indices at runtime.
#[allow(clippy::indexing_slicing)]
pub(crate) mod board;
mod component;
mod component_form;
mod component_form_util;
mod component_page;
mod dispatcher;
mod event_editor;
mod event_store;
mod event_todo_editor;
mod todo_editor;
mod todo_store;

pub use board::{BoardAction, BoardState, card_data_from_todo};
pub use event_todo_editor::EventOrTodoDraft;

use std::{cell::RefCell, error::Error, rc::Rc};

use aimcal_core::{Aim, Event, EventDraft, EventPatch, Kind, Todo, TodoDraft, TodoPatch};
use ratatui::Terminal;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::Backend;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::Dispatcher;
use crate::tui::event_editor::new_event_editor;
use crate::tui::event_store::EventStore;
use crate::tui::event_todo_editor::{EventTodoStore, new_event_todo_editor};
use crate::tui::todo_editor::new_todo_editor;
use crate::tui::todo_store::TodoStore;

pub fn draft_event(aim: &mut Aim, draft: EventDraft) -> Result<Option<EventDraft>, Box<dyn Error>> {
    let store = EventStore::from_draft(draft);
    let store = run_event_editor(aim, store)?;
    if store.submit {
        store.submit_draft(aim).map(Some)
    } else {
        Ok(None)
    }
}

pub fn patch_event(
    aim: &mut Aim,
    event: &impl Event,
    patch: EventPatch,
) -> Result<Option<EventPatch>, Box<dyn Error>> {
    let store = EventStore::from_patch(event, patch);
    let store = run_event_editor(aim, store)?;
    if store.submit {
        store.submit_patch(aim).map(Some)
    } else {
        Ok(None)
    }
}

pub fn draft_todo(aim: &mut Aim, draft: TodoDraft) -> Result<Option<TodoDraft>, Box<dyn Error>> {
    let store = TodoStore::from_draft(draft);
    let store = run_todo_editor(aim, store)?;
    if store.submit {
        store.submit_draft(aim).map(Some)
    } else {
        Ok(None)
    }
}

pub fn patch_todo(
    aim: &mut Aim,
    todo: &impl Todo,
    patch: TodoPatch,
) -> Result<Option<TodoPatch>, Box<dyn Error>> {
    let store = TodoStore::from_patch(todo, patch);
    let store = run_todo_editor(aim, store)?;
    if store.submit {
        store.submit_patch(aim).map(Some)
    } else {
        Ok(None)
    }
}

pub fn draft_event_or_todo(
    aim: &mut Aim,
    kind: Option<Kind>,
    event_draft: EventDraft,
    todo_draft: TodoDraft,
) -> Result<Option<EventOrTodoDraft>, Box<dyn Error>> {
    let store = EventTodoStore::new(kind, event_draft, todo_draft);
    let store = run_event_todo_editor(aim, store)?;
    if store.submit {
        store.submit_draft(aim).map(Some)
    } else {
        Ok(None)
    }
}

macro_rules! run_editor {
    ($fn: ident, $view: ident, $store: ident) => {
        fn $fn(aim: &mut Aim, store: $store) -> Result<$store, Box<dyn Error>> {
            let store = Rc::new(RefCell::new(store));

            let result = {
                let mut dispatcher = Dispatcher::new();
                $store::register_to(store.clone(), &mut dispatcher);

                ratatui::run(|mut terminal| {
                    let mut app = App::new($view(), dispatcher, &store, &mut terminal);

                    loop {
                        if let Err(e) = app.draw(&store, &mut terminal) {
                            break Err(e.into());
                        }

                        match app.read_event(&store) {
                            Err(e) => break Err(e),
                            Ok(Some(Message::Exit)) => break Ok(()),
                            Ok(_) => {} // Continue the loop to render the next frame
                        }
                    }
                })
            }; // release dispatcher and view here to avoid borrow conflicts
            aim.refresh_now(); // Ensure the current time is updated
            result?;

            let owned_store = Rc::try_unwrap(store)
                .map_err(|_| "Store still has references")?
                .into_inner();
            Ok(owned_store)
        }
    };
}

run_editor!(run_event_editor, new_event_editor, EventStore);
run_editor!(run_todo_editor, new_todo_editor, TodoStore);
run_editor!(run_event_todo_editor, new_event_todo_editor, EventTodoStore);

struct App<S, C: Component<S>> {
    dispatcher: Dispatcher,
    area: Rect, // TODO: support resize
    cursor_pos: Option<(u16, u16)>,
    view: C,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, C: Component<S>> App<S, C> {
    pub fn new<B: Backend>(
        mut view: C,
        mut dispatcher: Dispatcher,
        store: &RefCell<S>,
        terminal: &mut Terminal<B>,
    ) -> Self {
        let area = match terminal.size() {
            Ok(size) => Rect::new(0, 0, size.width, size.height),
            Err(_) => Rect::default(),
        };

        // Activate the first item
        view.activate(&mut dispatcher, store);

        Self {
            dispatcher,
            area,
            cursor_pos: view.get_cursor_position(store, area),
            view,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        store: &RefCell<S>,
        terminal: &mut Terminal<B>,
    ) -> Result<(), B::Error> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.view.render(store, frame.area(), frame.buffer_mut());

            if let Some(pos) = self.cursor_pos {
                frame.set_cursor_position(pos);
            }
        })?;
        Ok(())
    }

    pub fn read_event(&mut self, store: &RefCell<S>) -> Result<Option<Message>, Box<dyn Error>> {
        Ok(match event::read()? {
            event::Event::Key(e) if e.kind == KeyEventKind::Press => {
                // Handle key events for the current component
                match self.view.on_key(&mut self.dispatcher, store, self.area, e) {
                    Some(msg) => match msg {
                        Message::CursorUpdated => {
                            self.cursor_pos = self.view.get_cursor_position(store, self.area);
                            Some(Message::Handled)
                        }
                        _ => Some(msg),
                    },
                    None => None,
                }
            }
            _ => None, // Ignore other kinds of events
        })
    }
}

/// Run the Kanban board TUI with a poll-based event loop.
///
/// Unlike the `run_editor!` macro which uses blocking `event::read()`,
/// the board uses `event::poll()` with a 100ms timeout for:
/// - Auto-dismissing error messages after 3 seconds
/// - Terminal resize detection
pub fn run_board(aim: &mut Aim, initial_state: BoardState) -> Result<BoardState, Box<dyn Error>> {
    use board::Board;

    let store = Rc::new(RefCell::new(initial_state));

    let result = {
        let mut dispatcher = Dispatcher::new();

        ratatui::run(|terminal| {
            let mut view = Board::new();
            view.activate(&mut dispatcher, &store);

            loop {
                // Draw
                if let Err(e) = terminal.draw(|frame| {
                    view.render(&store, frame.area(), frame.buffer_mut());
                }) {
                    break Err(Box::<dyn Error>::from(e));
                }

                // Poll with 100ms timeout
                match event::poll(std::time::Duration::from_millis(100)) {
                    Ok(true) => {
                        if let event::Event::Key(e) =
                            event::read().unwrap_or_else(|_| event::Event::Key(KeyCode::Esc.into()))
                            && e.kind == KeyEventKind::Press
                        {
                            let area = terminal
                                .size()
                                .map(|s| Rect::new(0, 0, s.width, s.height))
                                .unwrap_or_default();
                            if let Some(Message::Exit) =
                                view.on_key(&mut dispatcher, &store, area, e)
                            {
                                break Ok(());
                            }
                        }
                    }
                    Ok(false) => {} // timeout, continue loop
                    Err(e) => break Err(Box::<dyn Error>::from(e)),
                }

                // Auto-clear error after 3 seconds
                {
                    let mut state = store.borrow_mut();
                    if let Some(ts) = state.error_timestamp
                        && ts.elapsed() > std::time::Duration::from_secs(3)
                    {
                        state.error_message = None;
                        state.error_timestamp = None;
                    }
                }
            }
        })
    }; // release dispatcher and view here to avoid borrow conflicts
    aim.refresh_now();
    result?;

    let owned = Rc::try_unwrap(store)
        .map_err(|_| "Store still has references")?
        .into_inner();
    Ok(owned)
}
