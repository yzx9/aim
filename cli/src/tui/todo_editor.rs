// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Aim, Id, Priority, Todo, TodoDraft, TodoPatch, TodoStatus};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;

use super::component::{Component, Form, FormItem, Input, Message, RadioGroup};
use crate::util::{format_datetime, parse_datetime};

/// TUI editor for editing todos.
pub struct TodoEditor {
    store: TodoStore,
    view: View,
}

impl TodoEditor {
    pub fn run_draft(aim: &mut Aim) -> Result<Option<TodoDraft>, Box<dyn Error>> {
        let draft = aim.default_todo_draft();
        let mut that = Self::new_with(Data {
            due: draft.due.map(format_datetime).unwrap_or_default(),
            priority: draft.priority.unwrap_or_default(),
            ..Data::default()
        });
        match that.run(aim)? {
            true => that.store.submit_draft().map(Some),
            false => Ok(None),
        }
    }

    pub async fn run_patch(aim: &mut Aim, uid: &Id) -> Result<Option<TodoPatch>, Box<dyn Error>> {
        let todo = aim.get_todo(uid).await?.ok_or("Todo not found")?;
        let mut that = Self::new_with(todo.into());
        match that.run(aim)? {
            true => that.store.submit_patch().map(Some),
            false => Ok(None),
        }
    }

    fn new_with(data: Data) -> Self {
        let fields = View::new();
        let store = TodoStore::new(data);
        Self {
            store,
            view: fields,
        }
    }

    fn run(&mut self, aim: &mut Aim) -> Result<bool, Box<dyn Error>> {
        let mut terminal = ratatui::init();
        self.view.setup(&self.store, &mut terminal);
        let result = loop {
            if let Err(e) = self.view.darw(&mut self.store, &mut terminal) {
                break Err(e);
            }

            match self.view.read_event(&mut self.store) {
                Err(e) => break Err(e),
                Ok(Some(Message::Exit)) => break Ok(false),
                Ok(Some(Message::Submit)) => break Ok(true),
                Ok(_) => {} // Continue the loop to render the next frame
            }
        };
        ratatui::restore();
        aim.refresh_now(); // Ensure the current time is updated
        result
    }
}

#[derive(Debug)]
struct TodoStore {
    data: Data,
    dirty: Marker,

    /// Whether to show verbose priority options
    verbose_priority: bool,
}

impl TodoStore {
    fn new(data: Data) -> Self {
        use Priority::*;
        let verbose_priority = matches!(data.priority, P1 | P3 | P4 | P6 | P7 | P9);
        Self {
            data,
            dirty: Marker::default(),
            verbose_priority,
        }
    }

    fn submit_draft(self) -> Result<TodoDraft, Box<dyn Error>> {
        Ok(TodoDraft {
            description: self.dirty.description.then_some(self.data.description),
            due: parse_datetime(&self.data.due)?,
            percent_complete: match self.dirty.percent_complete {
                true if self.data.percent_complete.is_empty() => None,
                true => Some(self.data.percent_complete.parse::<u8>()?),
                false => None,
            },
            priority: Some(self.data.priority), // Always commit since it was confirmed by the user
            status: Some(self.data.status),     // Always commit since it was confirmed by the user
            summary: if self.data.summary.is_empty() {
                "New todo".to_string()
            } else {
                self.data.summary
            },
        })
    }

    fn submit_patch(self) -> Result<TodoPatch, Box<dyn Error>> {
        Ok(TodoPatch {
            description: match self.dirty.description {
                true if self.data.description.is_empty() => Some(None),
                true => Some(Some(self.data.description.clone())),
                false => None,
            },
            due: match self.dirty.due {
                true => Some(parse_datetime(&self.data.due)?),
                false => None,
            },
            percent_complete: match self.dirty.percent_complete {
                true if self.data.percent_complete.is_empty() => Some(None),
                true => Some(Some(self.data.percent_complete.parse::<u8>()?)),
                false => None,
            },
            priority: self.dirty.priority.then_some(self.data.priority),
            status: self.dirty.status.then_some(self.data.status),
            summary: self.dirty.summary.then(|| self.data.summary.clone()),
        })
    }
}

#[derive(Debug, Default)]
struct Data {
    description: String,
    due: String,
    percent_complete: String,
    priority: Priority,
    status: TodoStatus,
    summary: String,
}

impl<T: Todo> From<T> for Data {
    fn from(todo: T) -> Self {
        Self {
            description: todo.description().unwrap_or("").to_owned(),
            due: todo.due().map(format_datetime).unwrap_or("".to_string()),
            percent_complete: todo
                .percent_complete()
                .map(|a| a.to_string())
                .unwrap_or("".to_string()),
            priority: todo.priority(),
            status: todo.status(),
            summary: todo.summary().to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct Marker {
    description: bool,
    due: bool,
    percent_complete: bool,
    priority: bool,
    status: bool,
    summary: bool,
}

struct View {
    area: Rect, // TODO: support resize
    cursor_pos: Option<(u16, u16)>,
    form: Form<TodoStore>,
}

impl View {
    pub fn new() -> Self {
        Self {
            area: Rect::default(),
            cursor_pos: None,
            form: Form::new(
                "Todo Editor".to_owned(),
                vec![
                    Box::new(new_summary()),
                    Box::new(new_due()),
                    Box::new(new_percent_complete()),
                    Box::new(FieldPriority::new()),
                    Box::new(new_status()),
                    Box::new(new_description()),
                ],
            ),
        }
    }

    pub fn setup<B: Backend>(&mut self, store: &TodoStore, terminal: &mut Terminal<B>) {
        if let Ok(size) = terminal.size() {
            self.area = Rect::new(0, 0, size.width, size.height);
        }

        // Activate the first item
        self.form.activate();
        if let Some(pos) = self.form.get_cursor_position(store, self.area) {
            self.cursor_pos = Some(pos)
        };
    }

    pub fn darw<B: Backend>(
        &mut self,
        store: &mut TodoStore,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            self.area = frame.area();
            self.render(store, frame.area(), frame.buffer_mut());

            if let Some(pos) = self.cursor_pos {
                frame.set_cursor_position(pos);
            }
        })?;
        Ok(())
    }

    pub fn read_event(&mut self, store: &mut TodoStore) -> Result<Option<Message>, Box<dyn Error>> {
        Ok(match event::read()? {
            Event::Key(e) if e.kind == KeyEventKind::Press => self.on_key(store, self.area, e.code),
            _ => None, // Ignore other kinds of events
        })
    }
}

impl Component<TodoStore> for View {
    fn render(&self, store: &TodoStore, area: Rect, buf: &mut Buffer) {
        self.form.render(store, area, buf);
    }

    fn on_key(&mut self, store: &mut TodoStore, area: Rect, key: KeyCode) -> Option<Message> {
        // Handle key events for the current component
        if let Some(msg) = self.form.on_key(store, area, key) {
            return match msg {
                Message::CursorUpdated => {
                    self.cursor_pos = self.form.get_cursor_position(store, area);
                    Some(Message::Handled)
                }
                _ => Some(msg),
            };
        }

        match key {
            KeyCode::Esc => Some(Message::Exit),
            _ => None,
        }
    }
}

macro_rules! new_input {
    ($name: ident, $title:expr, $field: ident) => {
        fn $name() -> Input<TodoStore> {
            Input::new(
                $title.to_string(),
                |store: &TodoStore| &store.data.$field,
                |store: &mut TodoStore, value: String| {
                    store.data.$field = value;
                    store.dirty.$field = true;
                },
            )
        }
    };
}

new_input!(new_summary, "Summary", summary);
new_input!(new_description, "Description", description);
new_input!(new_due, "Due", due);
new_input!(new_percent_complete, "Percent complete", percent_complete);

fn new_status() -> RadioGroup<TodoStore, TodoStatus> {
    use TodoStatus::*;
    let values = vec![NeedsAction, Completed, InProcess, Cancelled];
    let options = values.iter().map(ToString::to_string).collect();
    RadioGroup::new(
        "Status".to_string(),
        values,
        options,
        |store: &TodoStore| store.data.status,
        |store: &mut TodoStore, value: &TodoStatus| {
            store.data.status = *value;
            store.dirty.status = true;
        },
    )
}

#[derive(Debug)]
struct FieldPriority {
    verbose: RadioGroup<TodoStore, Priority>,
    concise: RadioGroup<TodoStore, Priority>,
}

impl FieldPriority {
    pub fn new() -> Self {
        use Priority::*;
        let values_verbose = vec![P1, P2, P3, P4, P5, P6, P7, P8, P9, None];
        let values_concise = vec![P2, P5, P8, None];

        let options_verbose = values_verbose
            .iter()
            .map(|a| Self::fmt(a, true).to_string())
            .collect();

        let options_concise = values_concise
            .iter()
            .map(|a| Self::fmt(a, false).to_string())
            .collect();

        fn get(store: &TodoStore) -> Priority {
            store.data.priority
        }

        fn set(store: &mut TodoStore, value: &Priority) {
            store.data.priority = *value;
            store.dirty.priority = true;
        }

        Self {
            verbose: RadioGroup::new(
                "Priority".to_string(),
                values_verbose,
                options_verbose,
                get,
                set,
            ),
            concise: RadioGroup::new(
                "Priority".to_string(),
                values_concise,
                options_concise,
                get,
                set,
            ),
        }
    }

    fn get(&self, store: &TodoStore) -> &RadioGroup<TodoStore, Priority> {
        match store.verbose_priority {
            true => &self.verbose,
            false => &self.concise,
        }
    }

    fn get_mut(&mut self, store: &TodoStore) -> &mut RadioGroup<TodoStore, Priority> {
        match store.verbose_priority {
            true => &mut self.verbose,
            false => &mut self.concise,
        }
    }

    fn fmt(priority: &Priority, verbose: bool) -> &'static str {
        match priority {
            Priority::P2 if !verbose => "HIGH",
            Priority::P5 if !verbose => "MID",
            Priority::P8 if !verbose => "LOW",
            Priority::None => "NONE",
            Priority::P1 => "1",
            Priority::P2 => "2",
            Priority::P3 => "3",
            Priority::P4 => "4",
            Priority::P5 => "5",
            Priority::P6 => "6",
            Priority::P7 => "7",
            Priority::P8 => "8",
            Priority::P9 => "9",
        }
    }
}

impl Component<TodoStore> for FieldPriority {
    fn render(&self, store: &TodoStore, area: Rect, buf: &mut Buffer) {
        self.get(store).render(store, area, buf)
    }

    fn on_key(&mut self, store: &mut TodoStore, area: Rect, key: KeyCode) -> Option<Message> {
        self.get_mut(store).on_key(store, area, key)
    }

    fn get_cursor_position(&self, store: &TodoStore, area: Rect) -> Option<(u16, u16)> {
        self.get(store).get_cursor_position(store, area)
    }
}

impl FormItem<TodoStore> for FieldPriority {
    fn activate(&mut self) {
        self.verbose.activate();
        self.concise.activate();
    }

    fn deactivate(&mut self) {
        self.verbose.deactivate();
        self.concise.deactivate();
    }
}
