// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::parser::{format_datetime, parse_datetime};
use aimcal_core::{Priority, Todo, TodoPatch, TodoStatus};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{self, Block, Paragraph, block},
};
use std::{error::Error, rc::Rc};
use unicode_width::UnicodeWidthStr;

/// TUI editor for editing todos.
#[derive(Debug)]
pub struct TodoEditor {
    store: Store,
    fields: Vec<Component>,
}

impl TodoEditor {
    pub fn new() -> Self {
        Self::new_with(Data {
            uid: "".to_string(),
            description: "".to_string(),
            due: "".to_string(),
            percent_complete: "".to_string(),
            priority: Priority::None,
            status: TodoStatus::NeedsAction,
            summary: "".to_string(),
        })
    }

    pub fn from(todo: impl Todo) -> Self {
        Self::new_with(todo.into())
    }

    fn new_with(data: Data) -> Self {
        let store = Store::new(data);
        let fields = vec![
            Component::Summary(FieldSummary::new(0)),
            Component::Due(FieldDue::new(1)),
            Component::PercentComplete(FieldPercentComplete::new(2)),
            Component::Priority(FieldPriority::new(3, &store)),
            Component::Status(FieldStatus::new(4)),
            Component::Description(FieldDescription::new(5)),
        ];
        Self { store, fields }
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
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_input(key.code),
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

        if let Some(a) = self.fields.get_mut(index) {
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
            (Some(field), Some(area)) => field.get_cursor_position(&self.store, *area),
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
            .white()
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

    fn update_priority(&mut self, value: Priority) {
        self.data.priority = value;
        self.dirty.priority = true;
    }

    fn update_status(&mut self, value: TodoStatus) {
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
            priority: self.dirty.priority.then_some(self.data.priority),
            status: self.dirty.status.then_some(self.data.status),
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

#[derive(Debug)]
struct Data {
    uid: String,
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
            uid: todo.uid().to_owned(),
            description: todo.description().unwrap_or("").to_owned(),
            due: todo.due().map(format_datetime).unwrap_or("".to_string()),
            percent_complete: todo
                .percent_complete()
                .map(|a| a.to_string())
                .unwrap_or("".to_string()),
            priority: todo.priority(),
            status: todo.status().unwrap_or(TodoStatus::NeedsAction),
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

    fn update(&mut self, store: &mut Store, action: Action) {
        match self {
            Component::Description(field) => field.update(store, action),
            Component::Due(field) => field.update(store, action),
            Component::PercentComplete(field) => field.update(store, action),
            Component::Priority(field) => field.update(store, action),
            Component::Status(field) => field.update(store, action),
            Component::Summary(field) => field.update(store, action),
        }
    }

    fn get_cursor_position(&self, store: &Store, area: Rect) -> Option<(u16, u16)> {
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

            pub fn get_cursor_position(&self, store: &Store, area: Rect) -> Option<(u16, u16)> {
                Some(
                    self.input(store)
                        .get_cursor_position(area, store.character_index),
                )
            }

            fn input<'a>(&self, store: &'a Store) -> Input<'a> {
                let value = store.data.$field.as_str();
                let active = store.field_index == self.index;
                Input {
                    title: $title,
                    value,
                    active,
                }
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
field_input!(FieldSummary, "Summary", summary, update_summary);

trait FieldSwitch {
    type T: Eq;

    fn get_title() -> &'static str;
    fn get_value(store: &Store) -> Self::T;
    fn update_value(store: &mut Store, value: &Self::T);

    fn index(&self) -> usize;
    fn values(&self) -> &Vec<Self::T>;
    fn options(&self) -> &Vec<String>;

    fn update(&mut self, store: &mut Store, action: Action) {
        match action {
            Action::MoveTo => {
                store.update_field_index(self.index());
                store.update_character_index(self.selected(store));
            }
            Action::MoveLeft => {
                let prev = (self.selected(store) + self.values().len() - 1) % self.values().len();
                store.update_character_index(prev);
                if let Some(a) = self.values().get(prev) {
                    Self::update_value(store, a);
                }
            }
            Action::MoveRight => {
                let next = (self.selected(store) + 1) % self.values().len();
                store.update_character_index(next);
                if let Some(a) = self.values().get(next) {
                    Self::update_value(store, a);
                }
            }
            _ => {}
        }
    }

    fn render(&self, store: &Store, area: Rect, buf: &mut Buffer) {
        self.switch(store).render(area, buf);
    }

    fn get_cursor_position(&self, store: &Store, area: Rect) -> Option<(u16, u16)> {
        self.switch(store)
            .get_cursor_position(area, store.character_index)
    }

    fn switch<'a>(&'a self, store: &'a Store) -> Switch<'a> {
        Switch {
            title: Self::get_title(),
            values: self.options(),
            active: store.field_index == self.index(),
            selected: self.selected(store),
        }
    }

    fn selected(&self, store: &Store) -> usize {
        let v = Self::get_value(store);
        self.values().iter().position(|s| s == &v).unwrap_or(0)
    }
}

#[derive(Debug)]
struct FieldPriority {
    index: usize,
    values: Vec<Priority>,
    options: Vec<String>,
}

impl FieldPriority {
    pub fn new(index: usize, store: &Store) -> Self {
        let verbose = Self::need_verbose(&store.data.priority);
        let (values, options) = Self::get_value_options(verbose);
        Self {
            index,
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
            Priority::P2 if !verbose => "high",
            Priority::P5 if !verbose => "mid",
            Priority::P8 if !verbose => "low",
            Priority::None => "none",
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
    fn update_value(store: &mut Store, value: &Self::T) { store.update_priority(*value); }

    fn index(&self) -> usize          { self.index }
    fn values(&self) -> &Vec<Self::T> { &self.values }
    fn options(&self) -> &Vec<String> { &self.options }
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
    fn update_value(store: &mut Store, value: &Self::T) { store.update_status(*value); }

    fn index(&self) -> usize          { self.index }
    fn values(&self) -> &Vec<Self::T> { &self.values }
    fn options(&self) -> &Vec<String> { &self.options }

}

struct Input<'a> {
    pub title: &'a str,
    pub value: &'a str,
    pub active: bool,
}

impl<'a> Input<'a> {
    pub fn get_cursor_position(&self, area: Rect, character_index: usize) -> (u16, u16) {
        let index = character_index.min(self.value.len());
        let width = self.value[0..index].width();
        let x = area.x + (width as u16) + 2; // border 1 + padding 1
        let y = area.y + 1; // title line: 1
        (x, y)
    }
}

impl Widget for &Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = title_block(self.title, self.active);
        Paragraph::new(self.value).block(block).render(area, buf);
    }
}

struct Switch<'a> {
    pub title: &'a str,
    pub values: &'a Vec<String>,
    pub active: bool,
    pub selected: usize,
}

impl<'a> Switch<'a> {
    pub fn get_cursor_position(&self, area: Rect, index: usize) -> Option<(u16, u16)> {
        let (_, inners) = self.split(area);
        inners.get(index).map(|area| {
            let x = area.x + 2; // 2 = padding (1) + active marker [ (1)
            (x, area.y)
        })
    }

    fn split(&self, area: Rect) -> (Block, Rc<[Rect]>) {
        let outer_block = title_block(self.title, self.active).white();
        let inner = outer_block.inner(area);
        let inner_blocks = self.layout().split(inner);
        (outer_block, inner_blocks)
    }

    fn layout(&self) -> Layout {
        let constraints = self
            .values
            .iter()
            // 6 = border left (1) + active marker [ ] (3) + space (1) + border right (1)
            .map(|s| Constraint::Min((6 + s.width()) as u16));

        Layout::horizontal(constraints)
    }
}

impl Widget for &Switch<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (outer, inners) = self.split(area);
        outer.render(area, buf);

        for (i, (value, area)) in self.values.iter().zip(inners.iter()).enumerate() {
            let label = format!("[{}] {}", if self.selected == i { 'x' } else { ' ' }, value);
            let inner_block = Block::default().padding(block::Padding::horizontal(1));
            Paragraph::new(label).block(inner_block).render(*area, buf);
        }
    }
}

fn title_block(title: &str, active: bool) -> Block {
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .borders(widgets::Borders::ALL)
        .padding(widgets::Padding::horizontal(1))
        .title_position(block::Position::Top)
        .title(format!(" {title} ").bold());

    match active {
        true => block.blue(),
        false => block.white(),
    }
}
