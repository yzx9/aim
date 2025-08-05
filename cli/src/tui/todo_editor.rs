// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Aim, Id, Priority, Todo, TodoDraft, TodoPatch, TodoStatus};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use super::component::{Input, Switch};
use crate::util::{format_datetime, parse_datetime};

/// TUI editor for editing todos.
pub struct TodoEditor {
    store: Store,
    fields: Fields,
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
        let fields = Fields::new();
        let store = Store::new(data);
        Self { store, fields }
    }

    fn run(&mut self, aim: &mut Aim) -> Result<bool, Box<dyn Error>> {
        let mut terminal = ratatui::init();

        // Send the initial position to the store
        if let Some(a) = self.fields.0.get_mut(0) {
            a.update(&mut self.store, Action::MoveTo)
        }

        let result = loop {
            if let Err(e) = terminal.draw(|frame| {
                self.store.cursor_pos = None;
                self.fields
                    .render(&mut self.store, frame.area(), frame.buffer_mut());

                if let Some(pos) = self.store.cursor_pos {
                    frame.set_cursor_position(pos);
                }
            }) {
                break Err(e.into());
            }

            match event::read() {
                Ok(e) => {
                    if let Some(summit) = self.handle_event(e) {
                        break Ok(summit);
                    }
                }
                Err(e) => break Err(e.into()),
            }
        };
        ratatui::restore();
        aim.refresh_now(); // Ensure the current time is updated
        result
    }

    fn handle_event(&mut self, event: Event) -> Option<bool> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let action = match key.code {
                    KeyCode::Up | KeyCode::BackTab => Action::MoveUp,
                    KeyCode::Down | KeyCode::Tab => Action::MoveDown,
                    KeyCode::Left => Action::MoveLeft,
                    KeyCode::Right => Action::MoveRight,
                    KeyCode::Backspace => Action::Delete,
                    KeyCode::Char(c) => Action::Char(c),
                    KeyCode::Enter => return Some(true), // Submit the form
                    KeyCode::Esc => return Some(false),  // Exit without submitting
                    _ => return None,
                };

                self.fields.update(&mut self.store, action);
                None
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Store {
    data: Data,
    dirty: Marker,
    field_index: usize,
    character_index: usize,
    cursor_pos: Option<(u16, u16)>,
}

impl Store {
    fn new(data: Data) -> Self {
        Self {
            data,
            dirty: Marker::default(),
            field_index: 0,
            character_index: 0,
            cursor_pos: None,
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

#[derive(Debug)]
enum Action {
    Char(char),
    Delete,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveTo,
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

trait Field {
    fn update(&mut self, store: &mut Store, action: Action);
    fn render(&mut self, store: &mut Store, area: Rect, buf: &mut Buffer);
}

struct Fields(Vec<Box<dyn Field>>);

impl Fields {
    pub fn new() -> Self {
        Self(vec![
            Box::new(FieldSummary::new(0)),
            Box::new(FieldDue::new(1)),
            Box::new(FieldPercentComplete::new(2)),
            Box::new(FieldPriority::new(3)),
            Box::new(FieldStatus::new(4)),
            Box::new(FieldDescription::new(5)),
        ])
    }
}

impl Field for Fields {
    fn render(&mut self, store: &mut Store, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Todo Editor ".bold());
        let instructions = Line::from(vec![
            " Prev ".into(),
            "<Up>".blue().bold(),
            " Next ".into(),
            "<Down>".blue().bold(),
            " Exit ".into(),
            "<Esc> ".blue().bold(),
        ]);
        Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED)
            .white()
            .render(area, buf);

        let layout = Layout::vertical(self.0.iter().map(|_| Constraint::Max(3))).margin(2);
        for (field, area) in self.0.iter_mut().zip(layout.split(area).iter()) {
            field.render(store, *area, buf);
        }
    }

    fn update(&mut self, store: &mut Store, action: Action) {
        let (index, action) = match action {
            Action::Char(_) => (store.field_index, action),
            Action::Delete => (store.field_index, action),
            Action::MoveUp => (
                (store.field_index + self.0.len() - 1) % self.0.len(),
                Action::MoveTo,
            ),
            Action::MoveDown => ((store.field_index + 1) % self.0.len(), Action::MoveTo),
            Action::MoveLeft => (store.field_index, Action::MoveLeft),
            Action::MoveRight => (store.field_index, Action::MoveRight),
            _ => return,
        };

        if let Some(a) = self.0.get_mut(index) {
            a.update(store, action)
        }
    }
}

macro_rules! field_input {
    ($name: ident, $title:expr, $field: ident) => {
        #[derive(Debug)]
        struct $name {
            index: usize,
        }

        impl $name {
            pub fn new(index: usize) -> Self {
                Self { index }
            }
        }

        impl Field for $name {
            fn update(&mut self, store: &mut Store, action: Action) {
                match action {
                    Action::Char(c) => {
                        let mut value = store.data.$field.to_owned();
                        value.insert(store.character_index, c);
                        store.data.$field = value;
                        store.dirty.$field = true;
                        store.character_index += 1;
                    }
                    Action::Delete if store.character_index > 0 => {
                        let mut value = store.data.$field.to_owned();
                        value.remove(store.character_index - 1);
                        store.data.$field = value;
                        store.dirty.$field = true;
                        store.character_index -= 1;
                    }
                    Action::MoveTo => {
                        store.field_index = self.index;
                        store.character_index = store.data.$field.len();
                    }
                    Action::MoveLeft if store.character_index > 0 => {
                        store.character_index -= 1;
                    }
                    Action::MoveRight if store.character_index < store.data.$field.len() => {
                        store.character_index += 1;
                    }
                    _ => {}
                }
            }

            fn render(&mut self, store: &mut Store, area: Rect, buf: &mut Buffer) {
                let value = store.data.$field.as_str();
                let active = store.field_index == self.index;
                let input = Input {
                    title: $title,
                    value,
                    active,
                };
                input.render(area, buf);

                if active {
                    let pos = input.get_cursor_position(area, store.character_index);
                    store.cursor_pos = Some(pos);
                }
            }
        }
    };
}

field_input!(FieldDescription, "Description", description);
field_input!(FieldDue, "Due", due);
field_input!(FieldPercentComplete, "Percent complete", percent_complete);
field_input!(FieldSummary, "Summary", summary);

trait FieldSwitch {
    type T: Eq;

    fn get_title() -> &'static str;
    fn get_value(store: &Store) -> Self::T;
    fn update_value(store: &mut Store, value: &Self::T);

    fn index(&self) -> usize;
    fn values(&self) -> &Vec<Self::T>;
    fn options(&self) -> &Vec<String>;

    /// Pre-render hook for the field. This method can be overridden by specific fields if needed.
    fn before_render(&mut self, _store: &mut Store) {}

    fn selected(&self, store: &Store) -> usize {
        let v = Self::get_value(store);
        self.values().iter().position(|s| s == &v).unwrap_or(0)
    }
}

impl<T: FieldSwitch> Field for T {
    fn update(&mut self, store: &mut Store, action: Action) {
        match action {
            Action::MoveTo => {
                store.field_index = self.index();
                store.character_index = self.selected(store);
            }
            Action::MoveLeft => {
                let prev = (self.selected(store) + self.values().len() - 1) % self.values().len();
                store.character_index = prev;
                if let Some(a) = self.values().get(prev) {
                    Self::update_value(store, a);
                }
            }
            Action::MoveRight => {
                let next = (self.selected(store) + 1) % self.values().len();
                store.character_index = next;
                if let Some(a) = self.values().get(next) {
                    Self::update_value(store, a);
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, store: &mut Store, area: Rect, buf: &mut Buffer) {
        Self::before_render(self, store);

        let active = store.field_index == self.index();
        let switch = Switch {
            title: Self::get_title(),
            values: self.options(),
            active,
            selected: self.selected(store),
        };
        switch.render(area, buf);

        if active {
            let pos = switch.get_cursor_position(area, store.character_index);
            store.cursor_pos = pos;
        }
    }
}

#[derive(Debug)]
struct FieldPriority {
    index: usize,
    verbose: bool,
    values: Vec<Priority>,
    options: Vec<String>,
}

impl FieldPriority {
    pub fn new(index: usize) -> Self {
        let (values, options) = Self::get_value_options(false);
        Self {
            index,
            verbose: false,
            values,
            options,
        }
    }

    fn need_verbose(priority: &Priority) -> bool {
        use Priority::*;
        matches!(priority, P1 | P3 | P4 | P6 | P7 | P9)
    }

    fn get_value_options(verbose: bool) -> (Vec<Priority>, Vec<String>) {
        use Priority::*;
        let values = match verbose {
            true => vec![P1, P2, P3, P4, P5, P6, P7, P8, P9, None],
            false => vec![P2, P5, P8, None],
        };

        let options = values
            .iter()
            .map(|a| Self::fmt(a, verbose).to_string())
            .collect();

        (values, options)
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

#[rustfmt::skip]
impl FieldSwitch for FieldPriority {
    type T = Priority;

    fn get_title() -> &'static str                      { "Priority" }
    fn get_value(store: &Store) -> Self::T              { store.data.priority }
    fn update_value(store: &mut Store, value: &Self::T) { 
        store.data.priority = *value; 
        store.dirty.priority = true;
    }

    fn index(&self) -> usize          { self.index }
    fn values(&self) -> &Vec<Self::T> { &self.values }
    fn options(&self) -> &Vec<String> { &self.options }

    fn before_render(&mut self, store: &mut Store) {
        let verbose = Self::need_verbose(&store.data.priority);
        if self.verbose != verbose {
            let (values, options) = Self::get_value_options(verbose);
            self.values = values;
            self.options = options;
        }
    }
}

#[derive(Debug)]
struct FieldStatus {
    index: usize,
    values: Vec<TodoStatus>,
    options: Vec<String>,
}

impl FieldStatus {
    pub fn new(index: usize) -> Self {
        use TodoStatus::*;
        let values = vec![NeedsAction, Completed, InProcess, Cancelled];
        let options = values.iter().map(ToString::to_string).collect();
        Self {
            index,
            values,
            options,
        }
    }
}

#[rustfmt::skip]
impl FieldSwitch for FieldStatus {
    type T = TodoStatus;

    fn get_title() -> &'static str                      { "Status" }
    fn get_value(store: &Store) -> Self::T              { store.data.status }
    fn update_value(store: &mut Store, value: &Self::T) {
        store.data.status = *value; 
        store.dirty.status = true;
    }

    fn index(&self) -> usize          { self.index }
    fn values(&self) -> &Vec<Self::T> { &self.values }
    fn options(&self) -> &Vec<String> { &self.options }
}
