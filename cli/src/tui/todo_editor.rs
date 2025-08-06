// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::rc::Rc;

use aimcal_core::{Aim, Id, Priority, Todo, TodoDraft, TodoPatch, TodoStatus};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use super::component::{Input, Switch};
use crate::util::{format_datetime, parse_datetime};

/// TUI editor for editing todos.
pub struct TodoEditor {
    store: TodoStore,
    view: Form,
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
        let fields = Form::new();
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
}

impl TodoStore {
    fn new(data: Data) -> Self {
        let dirty = Marker::default();
        Self { data, dirty }
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

trait Component {
    /// Renders the component into the given area.
    fn render(&mut self, store: &mut TodoStore, area: Rect, buf: &mut Buffer);

    /// Handles key events for the component.
    fn on_key(&mut self, _store: &mut TodoStore, _area: Rect, _key: KeyCode) -> Option<Message> {
        None // Default implementation does nothing
    }

    /// Returns the cursor position (row, column) for the component, if applicable.
    fn get_cursor_position(&self, _store: &TodoStore, _area: Rect) -> Option<(u16, u16)> {
        None // Default implementation returns no cursor position
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Message {
    Handled,
    CursorUpdated,
    Submit,
    Exit,
}

struct Form {
    items: Vec<Box<dyn FormItem>>,
    item_index: usize,
    area: Rect, // TODO: support resize
    cursor_pos: Option<(u16, u16)>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            items: vec![
                Box::new(FieldSummary::new()),
                Box::new(FieldDue::new()),
                Box::new(FieldPercentComplete::new()),
                Box::new(FieldPriority::new()),
                Box::new(FieldStatus::new()),
                Box::new(FieldDescription::new()),
            ],
            item_index: 0,
            area: Rect::default(),
            cursor_pos: None,
        }
    }

    pub fn setup<B: Backend>(&mut self, store: &TodoStore, terminal: &mut Terminal<B>) {
        if let Ok(size) = terminal.size() {
            self.area = Rect::new(0, 0, size.width, size.height);
        }

        // Activate the first item
        let areas = self.layout().split(self.area);
        self.activate_item(&areas, store);
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

    fn layout(&self) -> Layout {
        Layout::vertical(self.items.iter().map(|_| Constraint::Max(3))).margin(2)
    }

    fn activate_item(&mut self, areas: &Rc<[Rect]>, store: &TodoStore) {
        self.cursor_pos = self.items.get_mut(self.item_index).and_then(|a| {
            a.activate();

            areas
                .get(self.item_index)
                .and_then(|b| a.get_cursor_position(store, *b))
        });
    }
}

impl Component for Form {
    fn render(&mut self, store: &mut TodoStore, area: Rect, buf: &mut Buffer) {
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

        let areas = self.layout().split(area);
        for (field, area) in self.items.iter_mut().zip(areas.iter()) {
            field.render(store, *area, buf);
        }
    }

    fn on_key(&mut self, store: &mut TodoStore, area: Rect, key: KeyCode) -> Option<Message> {
        // Handle key events for the current component
        let areas = self.layout().split(area);
        if let Some((comp, subarea)) = self
            .items
            .iter_mut()
            .zip(areas.iter())
            .take(self.item_index + 1)
            .last()
        {
            if let Some(msg) = comp.on_key(store, *subarea, key) {
                return match msg {
                    Message::CursorUpdated => {
                        self.cursor_pos = comp.get_cursor_position(store, *subarea);
                        Some(Message::Handled)
                    }
                    _ => Some(msg),
                };
            }
        };

        match key {
            KeyCode::Down | KeyCode::Tab | KeyCode::Up | KeyCode::BackTab => {
                // deactivate current item
                if let Some(a) = self.items.get_mut(self.item_index) {
                    a.deactivate()
                }

                // move to next/previous item
                let offset = match key {
                    KeyCode::Down | KeyCode::Tab => 1,
                    KeyCode::Up | KeyCode::BackTab => self.items.len() - 1,
                    _ => 0,
                };
                self.item_index = (self.item_index + offset) % self.items.len();

                // activate new item
                self.activate_item(&areas, store);
                Some(Message::Handled)
            }
            KeyCode::Enter => Some(Message::Submit),
            KeyCode::Esc => Some(Message::Exit),
            _ => None,
        }
    }
}

trait FormItem: Component {
    fn activate(&mut self);
    fn deactivate(&mut self);
}

macro_rules! field_input {
    ($name: ident, $title:expr, $field: ident) => {
        #[derive(Debug)]
        struct $name {
            active: bool,
            character_index: usize,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    active: false,
                    character_index: 0,
                }
            }

            fn input<'a>(&self, store: &'a TodoStore) -> Input<'a> {
                let value = store.data.$field.as_str();
                Input {
                    title: $title,
                    value,
                    active: self.active,
                }
            }
        }

        impl Component for $name {
            fn render(&mut self, store: &mut TodoStore, area: Rect, buf: &mut Buffer) {
                self.input(store).render(area, buf);
            }

            fn on_key(
                &mut self,
                store: &mut TodoStore,
                _area: Rect,
                key: KeyCode,
            ) -> Option<Message> {
                if !self.active {
                    return None; // Only handle keys when the field is active
                }

                match key {
                    KeyCode::Left => {
                        if self.character_index > 0 {
                            self.character_index -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.character_index < store.data.$field.len() {
                            self.character_index += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if self.character_index > 0 {
                            let mut value = store.data.$field.to_owned();
                            value.remove(self.character_index - 1);
                            store.data.$field = value;
                            store.dirty.$field = true;
                            self.character_index -= 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        let mut value = store.data.$field.to_owned();
                        value.insert(self.character_index, c);
                        store.data.$field = value;
                        store.dirty.$field = true;
                        self.character_index += 1;
                    }
                    _ => return None,
                };

                // Always update the cursor position for simplicity
                Some(Message::CursorUpdated)
            }

            fn get_cursor_position(&self, store: &TodoStore, area: Rect) -> Option<(u16, u16)> {
                let pos = self
                    .input(store)
                    .get_cursor_position(area, self.character_index);
                Some(pos)
            }
        }

        impl FormItem for $name {
            fn activate(&mut self) {
                self.active = true;
                self.character_index = 0; // Reset character index when activated
            }

            fn deactivate(&mut self) {
                self.active = false;
                self.character_index = 0; // Reset character index when deactivated
            }
        }
    };
}

field_input!(FieldDescription, "Description", description);
field_input!(FieldDue, "Due", due);
field_input!(FieldPercentComplete, "Percent complete", percent_complete);
field_input!(FieldSummary, "Summary", summary);

trait FormRadioGroup {
    type T: Eq;

    fn get_title() -> &'static str;
    fn get_value(store: &TodoStore) -> Self::T;
    fn update_value(store: &mut TodoStore, value: &Self::T);

    fn active(&self) -> bool;
    fn update_active(&mut self, active: bool);

    fn values(&self) -> &Vec<Self::T>;
    fn options(&self) -> &Vec<String>;

    /// Pre-render hook for the field. This method can be overridden by specific fields if needed.
    fn before_render(&mut self, _store: &mut TodoStore) {}

    fn selected(&self, store: &TodoStore) -> usize {
        let v = Self::get_value(store);
        self.values().iter().position(|s| s == &v).unwrap_or(0)
    }

    fn switch(&self, store: &TodoStore) -> Switch {
        Switch {
            title: Self::get_title(),
            values: self.options(),
            active: self.active(),
            selected: self.selected(store),
        }
    }
}

impl<T: FormRadioGroup> Component for T {
    fn render(&mut self, store: &mut TodoStore, area: Rect, buf: &mut Buffer) {
        Self::before_render(self, store);
        self.switch(store).render(area, buf);
    }

    fn on_key(&mut self, store: &mut TodoStore, _area: Rect, key: KeyCode) -> Option<Message> {
        if !self.active() {
            return None; // Only handle keys when the field is active
        }

        match key {
            KeyCode::Left | KeyCode::Right => {
                let offset = match key {
                    KeyCode::Left => self.values().len() - 1,
                    KeyCode::Right => 1,
                    _ => 0,
                };
                let index = (self.selected(store) + offset) % self.values().len();

                match self.values().get(index) {
                    Some(a) => {
                        Self::update_value(store, a);
                        Some(Message::CursorUpdated)
                    }
                    None => Some(Message::Handled),
                }
            }
            _ => None,
        }
    }

    fn get_cursor_position(&self, store: &TodoStore, area: Rect) -> Option<(u16, u16)> {
        self.switch(store)
            .get_cursor_position(area, self.selected(store))
    }
}

impl<T: FormRadioGroup> FormItem for T {
    fn activate(&mut self) {
        self.update_active(true);
    }

    fn deactivate(&mut self) {
        self.update_active(false);
    }
}

#[derive(Debug)]
struct FieldPriority {
    active: bool,
    verbose: bool,
    values: Vec<Priority>,
    options: Vec<String>,
}

impl FieldPriority {
    pub fn new() -> Self {
        let (values, options) = Self::get_value_options(false);
        Self {
            active: false,
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

impl FormRadioGroup for FieldPriority {
    type T = Priority;

    fn get_title() -> &'static str {
        "Priority"
    }

    fn get_value(store: &TodoStore) -> Self::T {
        store.data.priority
    }

    fn update_value(store: &mut TodoStore, value: &Self::T) {
        store.data.priority = *value;
        store.dirty.priority = true;
    }

    fn active(&self) -> bool {
        self.active
    }

    fn update_active(&mut self, active: bool) {
        self.active = active;
    }

    fn values(&self) -> &Vec<Self::T> {
        &self.values
    }

    fn options(&self) -> &Vec<String> {
        &self.options
    }

    fn before_render(&mut self, store: &mut TodoStore) {
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
    active: bool,
    values: Vec<TodoStatus>,
    options: Vec<String>,
}

impl FieldStatus {
    pub fn new() -> Self {
        use TodoStatus::*;
        let values = vec![NeedsAction, Completed, InProcess, Cancelled];
        let options = values.iter().map(ToString::to_string).collect();
        Self {
            active: false,
            values,
            options,
        }
    }
}

#[rustfmt::skip]
impl FormRadioGroup for FieldStatus {
    type T = TodoStatus;

    fn get_title() -> &'static str                          { "Status" }
    fn get_value(store: &TodoStore) -> Self::T              { store.data.status }
    fn update_value(store: &mut TodoStore, value: &Self::T) {
        store.data.status = *value; 
        store.dirty.status = true;
    }

    fn active(&self) -> bool                  { self.active }
    fn update_active(&mut self, active: bool) { self.active = active; }

    fn values(&self) -> &Vec<Self::T> { &self.values }
    fn options(&self) -> &Vec<String> { &self.options }
}
