// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Config,
    parser::{format_datetime, parse_datetime},
};
use aimcal_core::{Aim, Priority, Todo, TodoDraft, TodoPatch, TodoStatus};
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
    fields: Fields,
}

impl TodoEditor {
    pub fn run_draft(config: &Config, aim: &mut Aim) -> Result<Option<TodoDraft>, Box<dyn Error>> {
        let mut that = Self::new_with(Data {
            due: config
                .core
                .default_due
                .map(|a| (aim.now() + a).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default(),
            priority: config.core.default_priority,
            ..Data::default()
        });

        let should_submit = that.run()?;
        let result = match should_submit {
            true => that.store.submit_draft().map(Some),
            false => Ok(None),
        };
        aim.refresh_now(); // Ensure the current time is updated
        result
    }

    pub async fn run_patch(aim: &mut Aim, uid: &str) -> Result<Option<TodoPatch>, Box<dyn Error>> {
        let todo = aim.get_todo(uid).await?.ok_or("Todo not found")?;
        let mut that = Self::new_with(todo.into());

        let should_submit = that.run()?;
        let result = match should_submit {
            true => that.store.submit_patch().map(Some),
            false => Ok(None),
        };
        aim.refresh_now(); // Ensure the current time is updated
        result
    }

    fn new_with(data: Data) -> Self {
        let fields = Fields::new();
        let store = Store::new(data);
        Self { store, fields }
    }

    fn run(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut terminal = ratatui::init();

        // Send the initial position to the store
        if let Some(a) = self.fields.0.get_mut(0) {
            a.update(&mut self.store, Action::MoveTo)
        }

        let result = loop {
            if let Err(e) = terminal.draw(|frame| {
                self.store.update_cursor_position(None);
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
        result
    }

    fn handle_event(&mut self, event: Event) -> Option<bool> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_input(key.code),
            _ => None,
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<bool> {
        let action = match key {
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
}

#[derive(Debug)]
struct Store {
    data: Data,
    dirty: Marker,
    field_index: usize,
    character_index: usize,
    cursor_pos: Option<(u16, u16)>,
}

macro_rules! define_update {
    ($name: ident, $field: ident, $type: ty) => {
        fn $name(&mut self, value: $type) {
            self.data.$field = value;
            self.dirty.$field = true;
        }
    };
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

    define_update!(update_description, description, String);
    define_update!(update_due, due, String);
    define_update!(update_percent_complete, percent_complete, String);
    define_update!(update_priority, priority, Priority);
    define_update!(update_status, status, TodoStatus);
    define_update!(update_summary, summary, String);

    fn update_field_index(&mut self, index: usize) {
        self.field_index = index;
    }

    fn update_character_index(&mut self, index: usize) {
        self.character_index = index;
    }

    fn update_cursor_position(&mut self, pos: Option<(u16, u16)>) {
        self.cursor_pos = pos;
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

#[derive(Debug)]
struct Fields(Vec<Field>);

impl Fields {
    pub fn new() -> Self {
        Self(vec![
            Field::Summary(FieldSummary::new(0)),
            Field::Due(FieldDue::new(1)),
            Field::PercentComplete(FieldPercentComplete::new(2)),
            Field::Priority(FieldPriority::new(3)),
            Field::Status(FieldStatus::new(4)),
            Field::Description(FieldDescription::new(5)),
        ])
    }

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

#[derive(Debug)]
enum Field {
    Description(FieldDescription),
    Due(FieldDue),
    PercentComplete(FieldPercentComplete),
    Priority(FieldPriority),
    Status(FieldStatus),
    Summary(FieldSummary),
}

impl Field {
    fn render(&mut self, store: &mut Store, area: Rect, buf: &mut Buffer) {
        match self {
            Field::Description(field) => field.render(store, area, buf),
            Field::Due(field) => field.render(store, area, buf),
            Field::PercentComplete(field) => field.render(store, area, buf),
            Field::Priority(field) => field.render(store, area, buf),
            Field::Status(field) => field.render(store, area, buf),
            Field::Summary(field) => field.render(store, area, buf),
        }
    }

    fn update(&mut self, store: &mut Store, action: Action) {
        match self {
            Field::Description(field) => field.update(store, action),
            Field::Due(field) => field.update(store, action),
            Field::PercentComplete(field) => field.update(store, action),
            Field::Priority(field) => field.update(store, action),
            Field::Status(field) => field.update(store, action),
            Field::Summary(field) => field.update(store, action),
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
                    Action::Char(c) => {
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

            pub fn render(&self, store: &mut Store, area: Rect, buf: &mut Buffer) {
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
                    store.update_cursor_position(Some(pos));
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

    /// Pre-render hook for the field. This method can be overridden by specific fields if needed.
    fn before_render(&mut self, _store: &mut Store) {}

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
            store.update_cursor_position(pos);
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
    verbose: bool,
    values: Vec<Priority>,
    options: Vec<String>,
}

impl FieldPriority {
    pub fn new(index: usize) -> Self {
        let verbose = false;
        let (values, options) = Self::get_value_options(false);
        Self {
            index,
            verbose,
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
    fn update_value(store: &mut Store, value: &Self::T) { store.update_priority(*value); }

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
        let outer_block = title_block(self.title, self.active);
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
