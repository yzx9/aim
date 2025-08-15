// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, rc::Rc};

use aimcal_core::{Aim, Event, EventDraft, EventPatch, EventStatus, Todo, TodoDraft, TodoPatch};
use ratatui::Terminal;
use ratatui::crossterm::event::{self, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::prelude::Backend;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::Dispatcher;
use crate::tui::event_editor::EventEditor;
use crate::tui::event_store::EventStore;
use crate::tui::event_todo_editor::{EventOrTodoDraft, EventTodoEditor, EventTodoStore};
use crate::tui::todo_editor::TodoEditor;
use crate::tui::todo_store::TodoStore;

pub fn draft_event(aim: &mut Aim) -> Result<Option<EventDraft>, Box<dyn Error>> {
    let store = EventStore::new_by_draft(EventDraft {
        description: None,
        end: None,
        start: None,
        status: EventStatus::default(),
        summary: String::new(),
    });
    let store = run_event_editor(aim, store)?;
    match store.submit {
        true => store.submit_draft().map(Some),
        false => Ok(None),
    }
}

pub fn patch_event(
    aim: &mut Aim,
    event: &impl Event,
) -> Result<Option<EventPatch>, Box<dyn Error>> {
    let store = EventStore::new_by_event(event);
    let store = run_event_editor(aim, store)?;
    match store.submit {
        true => store.submit_patch().map(Some),
        false => Ok(None),
    }
}

pub fn draft_todo(aim: &mut Aim) -> Result<Option<TodoDraft>, Box<dyn Error>> {
    let draft = aim.default_todo_draft();
    let store = TodoStore::new_by_draft(draft);
    let store = run_todo_editor(aim, store)?;
    match store.submit {
        true => store.submit_draft().map(Some),
        false => Ok(None),
    }
}

pub fn patch_todo(aim: &mut Aim, todo: &impl Todo) -> Result<Option<TodoPatch>, Box<dyn Error>> {
    let store = TodoStore::new_by_todo(todo);
    let store = run_todo_editor(aim, store)?;
    match store.submit {
        true => store.submit_patch().map(Some),
        false => Ok(None),
    }
}

pub fn draft_event_or_todo(aim: &mut Aim) -> Result<Option<EventOrTodoDraft>, Box<dyn Error>> {
    let store = EventTodoStore::new(aim.default_event_draft(), aim.default_todo_draft());
    let store = run_event_todo_editor(aim, store)?;
    match store.submit {
        true => store.submit_draft().map(Some),
        false => Ok(None),
    }
}

macro_rules! run_editor {
    ($fn: ident, $view: ident, $store: ident) => {
        fn $fn(aim: &mut Aim, store: $store) -> Result<$store, Box<dyn Error>> {
            let store = Rc::new(RefCell::new(store));

            let mut terminal = ratatui::init();
            let result = {
                let mut dispatcher = Dispatcher::new();
                $store::register_to(store.clone(), &mut dispatcher);
                let mut app = App::new($view::new(), dispatcher, &store, &mut terminal);

                loop {
                    if let Err(e) = app.darw(&store, &mut terminal) {
                        break Err(e);
                    }

                    match app.read_event(&store) {
                        Err(e) => break Err(e),
                        Ok(Some(Message::Exit)) => break Ok(()),
                        Ok(_) => {} // Continue the loop to render the next frame
                    }
                }
            }; // release dispatcher and view here to avoid borrow conflicts
            ratatui::restore();
            aim.refresh_now(); // Ensure the current time is updated
            result?;

            let owned_store = Rc::try_unwrap(store)
                .map_err(|_| "Store still has references")?
                .into_inner();
            Ok(owned_store)
        }
    };
}

run_editor!(run_event_editor, EventEditor, EventStore);
run_editor!(run_todo_editor, TodoEditor, TodoStore);
run_editor!(run_event_todo_editor, EventTodoEditor, EventTodoStore);

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
        store: &Rc<RefCell<S>>,
        terminal: &mut Terminal<B>,
    ) -> Self {
        let area = match terminal.size() {
            Ok(size) => Rect::new(0, 0, size.width, size.height),
            Err(_) => Rect::default(),
        };

        // Activate the first item
        view.activate(&mut dispatcher);

        Self {
            dispatcher,
            area,
            cursor_pos: view.get_cursor_position(store, area),
            view,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn darw<B: Backend>(
        &mut self,
        store: &Rc<RefCell<S>>,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.view.render(store, frame.area(), frame.buffer_mut());

            if let Some(pos) = self.cursor_pos {
                frame.set_cursor_position(pos);
            }
        })?;
        Ok(())
    }

    pub fn read_event(
        &mut self,
        store: &Rc<RefCell<S>>,
    ) -> Result<Option<Message>, Box<dyn Error>> {
        Ok(match event::read()? {
            event::Event::Key(e) if e.kind == KeyEventKind::Press => {
                // Handle key events for the current component
                let (form, dispatcher, area) = (&mut self.view, &mut self.dispatcher, self.area);
                match form.on_key(dispatcher, store, self.area, e.code) {
                    Some(msg) => match msg {
                        Message::CursorUpdated => {
                            self.cursor_pos = self.view.get_cursor_position(store, area);
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
