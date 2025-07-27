// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::parser::{format_datetime, parse_datetime};
use aimcal_core::{Priority, Todo, TodoPatch, TodoStatus};
use clap::ValueEnum;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{self, Block, Paragraph, block},
};
use std::error::Error;
use unicode_width::UnicodeWidthStr;

/// TUI editor for editing todos.
#[derive(Debug)]
pub struct TodoEditor {
    store: Store,
    fields: Vec<Component>,
}

impl TodoEditor {
    pub fn new() -> Self {
        Self::new_with(Data::default())
    }

    pub fn from(todo: impl Todo) -> Self {
        Self::new_with(todo.into())
    }

    fn new_with(data: Data) -> Self {
        Self {
            store: Store::new(data),
            fields: vec![
                Component::Summary(FieldSummary::new(0)),
                Component::Due(FieldDue::new(1)),
                Component::PercentComplete(FieldPercentComplete::new(2)),
                Component::Priority(FieldPriority::new(3)),
                Component::Status(FieldStatus::new(4)),
                Component::Description(FieldDescription::new(5)),
            ],
        }
    }

    pub fn run(mut self) -> Result<Option<TodoPatch>, Box<dyn Error>> {
        let mut terminal = ratatui::init();
        let result = loop {
            if let Err(e) = terminal.draw(|frame| self.draw(frame)) {
                break Err(e.into());
            }

            match event::read() {
                Ok(e) => {
                    if let Some(summit) = self.handle_event(e) {
                        break match summit {
                            true => self.store.submit().map(Some),
                            false => Ok(None),
                        };
                    }
                }
                Err(e) => break Err(e.into()),
            }
        };
        ratatui::restore();
        result
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(self, area);

        if let Some(cursor_pos) = self.get_cursor_position(area) {
            frame.set_cursor_position(cursor_pos);
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<bool> {
        match event {
            Event::Key(key) => self.handle_input(key.code),
            _ => None,
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<bool> {
        let (index, action) = match key {
            KeyCode::Up | KeyCode::BackTab => (
                (self.store.field_index + self.fields.len() - 1) % self.fields.len(),
                Action::MoveTo,
            ),
            KeyCode::Down | KeyCode::Tab => (
                (self.store.field_index + 1) % self.fields.len(),
                Action::MoveTo,
            ),
            KeyCode::Left => (self.store.field_index, Action::MoveLeft),
            KeyCode::Right => (self.store.field_index, Action::MoveRight),
            KeyCode::Backspace => (self.store.field_index, Action::Delete),
            KeyCode::Char(c) => (self.store.field_index, Action::Typing { c }),
            KeyCode::Enter => {
                return Some(true); // Submit the form
            }
            KeyCode::Esc => {
                return Some(false); // Exit without submitting
            }
            _ => {
                return None;
            }
        };

        if let Some(a) = self.fields.get(index) {
            a.update(&mut self.store, action)
        }
        None
    }

    fn layout(&self) -> Layout {
        Layout::vertical(self.fields.iter().map(|_| Constraint::Max(3))).margin(2)
    }

    fn get_cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let areas = self.layout().split(area);
        let index = self.store.field_index;
        match (self.fields.get(index), areas.get(index)) {
            (Some(field), Some(area)) => Some(field.get_cursor_position(&self.store, *area)),
            _ => None,
        }
    }
}

impl Default for TodoEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &TodoEditor {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
            .render(area, buf);

        for (field, area) in self.fields.iter().zip(self.layout().split(area).iter()) {
            field.render(&self.store, *area, buf);
        }
    }
}

#[derive(Debug)]
struct Store {
    data: Data,
    dirty: Marker,
    field_index: usize,
    character_index: usize,
}

impl Store {
    fn new(data: Data) -> Self {
        let character_index = data.summary.len();
        Self {
            data,
            dirty: Marker {
                description: false,
                due: false,
                percent_complete: false,
                priority: false,
                status: false,
                summary: false,
            },
            field_index: 0,
            character_index,
        }
    }

    fn update_description(&mut self, value: String) {
        self.data.description = value;
        self.dirty.description = true;
    }

    fn update_due(&mut self, value: String) {
        self.data.due = value;
        self.dirty.due = true;
    }

    fn update_percent_complete(&mut self, value: String) {
        self.data.percent_complete = value;
        self.dirty.percent_complete = true;
    }

    fn update_priority(&mut self, value: String) {
        self.data.priority = value;
        self.dirty.priority = true;
    }

    fn update_status(&mut self, value: String) {
        self.data.status = value;
        self.dirty.status = true;
    }

    fn update_summary(&mut self, value: String) {
        self.data.summary = value;
        self.dirty.summary = true;
    }

    fn update_field_index(&mut self, index: usize) {
        self.field_index = index;
    }

    fn update_character_index(&mut self, index: usize) {
        self.character_index = index;
    }

    fn submit(self) -> Result<TodoPatch, Box<dyn Error>> {
        const IGNORE_CASE: bool = false;
        Ok(TodoPatch {
            uid: self.data.uid,
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
            priority: match self.dirty.priority {
                true => Some(Priority::from_str(&self.data.priority, IGNORE_CASE)?),
                false => None,
            },
            status: match self.dirty.status {
                true => Some(TodoStatus::from_str(&self.data.status, IGNORE_CASE)?),
                false => None,
            },
            summary: self.dirty.summary.then(|| self.data.summary.clone()),
        })
    }
}

#[derive(Debug)]
enum Action {
    Typing { c: char },
    Delete,
    MoveTo,
    MoveLeft,
    MoveRight,
}

#[derive(Debug, Default)]
struct Data {
    uid: String,
    description: String,
    due: String,
    percent_complete: String,
    priority: String,
    status: String,
    summary: String,
}

impl<T: Todo> From<T> for Data {
    fn from(todo: T) -> Self {
        Self {
            uid: todo.uid().to_owned(),
            description: todo.description().unwrap_or("").to_owned(),
            due: todo.due().map(format_datetime).unwrap_or("".to_string()),
            percent_complete: todo
                .percent_complete()
                .map(|a| a.to_string())
                .unwrap_or("".to_string()),
            priority: match todo.priority() {
                Priority::None => "none",
                Priority::P1 => "1",
                Priority::P2 => "2",
                Priority::P3 => "3",
                Priority::P4 => "4",
                Priority::P5 => "3",
                Priority::P6 => "6",
                Priority::P7 => "7",
                Priority::P8 => "8",
                Priority::P9 => "9",
            }
            .to_string(),
            status: todo
                .status()
                .map(|a| a.to_string())
                .unwrap_or("".to_string()),
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

#[derive(Debug)]
enum Component {
    Description(FieldDescription),
    Due(FieldDue),
    PercentComplete(FieldPercentComplete),
    Priority(FieldPriority),
    Status(FieldStatus),
    Summary(FieldSummary),
}

impl Component {
    fn render(&self, store: &Store, area: Rect, buf: &mut Buffer) {
        match self {
            Component::Description(field) => field.render(store, area, buf),
            Component::Due(field) => field.render(store, area, buf),
            Component::PercentComplete(field) => field.render(store, area, buf),
            Component::Priority(field) => field.render(store, area, buf),
            Component::Status(field) => field.render(store, area, buf),
            Component::Summary(field) => field.render(store, area, buf),
        }
    }

    fn update(&self, store: &mut Store, action: Action) {
        match self {
            Component::Description(field) => field.update(store, action),
            Component::Due(field) => field.update(store, action),
            Component::PercentComplete(field) => field.update(store, action),
            Component::Priority(field) => field.update(store, action),
            Component::Status(field) => field.update(store, action),
            Component::Summary(field) => field.update(store, action),
        }
    }

    fn get_cursor_position(&self, store: &Store, area: Rect) -> (u16, u16) {
        match self {
            Component::Description(field) => field.get_cursor_position(store, area),
            Component::Due(field) => field.get_cursor_position(store, area),
            Component::PercentComplete(field) => field.get_cursor_position(store, area),
            Component::Priority(field) => field.get_cursor_position(store, area),
            Component::Status(field) => field.get_cursor_position(store, area),
            Component::Summary(field) => field.get_cursor_position(store, area),
        }
    }
}

macro_rules! field_input {
    ($name: ident, $title:expr, $field: ident, $update_call: ident) => {
        #[derive(Debug)]
        struct $name {
            index: usize,
        }

        impl $name {
            pub fn new(index: usize) -> Self {
                Self { index }
            }

            fn update(&self, store: &mut Store, action: Action) {
                match action {
                    Action::Typing { c } if store.character_index > 0 => {
                        let mut value = store.data.$field.to_owned();
                        value.insert(store.character_index, c);
                        store.$update_call(value);
                        store.update_character_index(store.character_index + 1);
                    }
                    Action::Delete if store.character_index > 0 => {
                        let mut value = store.data.$field.to_owned();
                        value.remove(store.character_index - 1);
                        store.$update_call(value);
                        store.update_character_index(store.character_index - 1);
                    }
                    Action::MoveTo => {
                        store.update_field_index(self.index);
                        store.update_character_index(store.data.$field.len());
                    }
                    Action::MoveLeft if store.character_index > 0 => {
                        store.update_character_index(store.character_index - 1);
                    }
                    Action::MoveRight if store.character_index < store.data.$field.len() => {
                        store.update_character_index(store.character_index + 1);
                    }
                    _ => {}
                }
            }

            pub fn render(&self, store: &Store, area: Rect, buf: &mut Buffer) {
                self.input(store).render(area, buf);
            }

            pub fn get_cursor_position(&self, store: &Store, area: Rect) -> (u16, u16) {
                self.input(store)
                    .get_cursor_position(area, store.character_index)
            }

            fn input<'a>(&self, store: &'a Store) -> Input<'a> {
                let value = store.data.$field.as_str();
                let active = store.field_index == self.index;
                Input::new($title, value, active)
            }
        }
    };
}

field_input!(
    FieldDescription,
    "Description",
    description,
    update_description
);
field_input!(FieldDue, "Due", due, update_due);
field_input!(
    FieldPercentComplete,
    "Percent complete",
    percent_complete,
    update_percent_complete
);
field_input!(FieldPriority, "Priority", priority, update_priority);
field_input!(FieldStatus, "Status", status, update_status);
field_input!(FieldSummary, "Summary", summary, update_summary);

struct Input<'a> {
    title: &'a str,
    value: &'a str,
    active: bool,
}

impl<'a> Input<'a> {
    fn new(title: &'a str, value: &'a str, active: bool) -> Self {
        Self {
            title,
            value,
            active,
        }
    }

    fn get_cursor_position(&self, area: Rect, character_index: usize) -> (u16, u16) {
        let index = character_index.min(self.value.len());
        let width = self.value[0..index].width();
        let x = area.x + (width as u16) + 2; // border 1 + padding 1
        let y = area.y + 1; // title line: 1
        (x, y)
    }
}

impl Widget for &Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .borders(widgets::Borders::ALL)
            .padding(widgets::Padding::horizontal(1))
            .title_position(block::Position::Top)
            .title(format!(" {} ", self.title).bold());

        let block = match self.active {
            true => block.blue(),
            false => block.gray(),
        };

        Paragraph::new(self.value).block(block).render(area, buf);
    }
}

