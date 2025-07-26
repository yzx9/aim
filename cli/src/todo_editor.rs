// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cmd_todo::ArgTodoStatus,
    parser::{ParsedPriority, parse_datetime},
    todo_formatter::TodoColumnDue,
};
use aimcal_core::{Todo, TodoPatch};
use clap::ValueEnum;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{Block, Padding, Paragraph},
};
use std::error::Error;

/// TUI editor for editing todos.
#[derive(Debug)]
pub struct TodoEditor {
    data: Data,
    dirty: Dirty,
    fields: Vec<TodoEditorField>,
    field_index: usize,
    character_index: usize,
}

impl TodoEditor {
    pub fn new() -> Self {
        Self::new_with(Data::default())
    }

    pub fn from(todo: impl Todo) -> Self {
        Self::new_with(Data {
            uid: todo.uid().to_owned(),
            description: todo.description().unwrap_or("").to_owned(),
            due: todo
                .due()
                .map(TodoColumnDue::format_value)
                .unwrap_or("".to_string()),
            percent_complete: todo
                .percent_complete()
                .map(|a| a.to_string())
                .unwrap_or("".to_string()),
            priority: ParsedPriority::from(&todo.priority()).to_string(),
            status: todo
                .status()
                .map(|a| ArgTodoStatus::from(a).to_string())
                .unwrap_or("".to_string()),
            summary: todo.summary().to_string(),
        })
    }

    pub fn run(mut self) -> Result<Option<TodoPatch>, Box<dyn Error>> {
        let mut terminal = ratatui::init();

        let result = loop {
            terminal.draw(|frame| {
                frame.render_widget(&self, frame.area());

                let areas = Layout::vertical(self.fields.iter().map(|_| Constraint::Max(3)))
                    .margin(2)
                    .split(frame.area());

                if let Some(active_area) = areas.get(self.field_index) {
                    let x = active_area.x + self.character_index as u16 + 2;
                    let y = active_area.y + 1;
                    frame.set_cursor_position((x, y))
                }
            })?;

            if let Event::Key(key) = event::read()? {
                if let Some(summit) = self.handle_input(key.code) {
                    break Ok(if summit { Some(self.submit()?) } else { None });
                }
            }
        };

        ratatui::restore();
        result
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<bool> {
        use TodoEditorField::*;
        match key {
            KeyCode::Enter => {
                return Some(true); // Submit the form
            }
            KeyCode::Esc => {
                return Some(false); // Exit without submitting
            }
            KeyCode::Up | KeyCode::BackTab => {
                let len = self.fields.len();
                self.field_index = (self.field_index + len - 1) % len;
                self.character_index = match self.fields[self.field_index] {
                    Description => self.data.description.len(),
                    Due => self.data.due.len(),
                    PercentComplete => self.data.percent_complete.len(),
                    Priority => self.data.priority.len(),
                    Status => self.data.status.len(),
                    Summary => self.data.summary.len(),
                };
            }
            KeyCode::Down | KeyCode::Tab => {
                self.field_index = (self.field_index + 1) % self.fields.len();
                self.character_index = match self.fields[self.field_index] {
                    Description => self.data.description.len(),
                    Due => self.data.due.len(),
                    PercentComplete => self.data.percent_complete.len(),
                    Priority => self.data.priority.len(),
                    Status => self.data.status.len(),
                    Summary => self.data.summary.len(),
                };
            }
            KeyCode::Left => {
                self.character_index = self.character_index.saturating_sub(1);
            }
            KeyCode::Right => {
                let len = match self.fields[self.field_index] {
                    Description => self.data.description.len(),
                    Due => self.data.due.len(),
                    PercentComplete => self.data.percent_complete.len(),
                    Priority => self.data.priority.len(),
                    Status => self.data.status.len(),
                    Summary => self.data.summary.len(),
                };
                self.character_index = self.character_index.saturating_add(1).min(len);
            }
            KeyCode::Backspace => {
                if self.character_index > 0 {
                    let current_index = self.character_index;
                    match self.fields.get(self.field_index) {
                        Some(Description) => {
                            self.data.description.remove(current_index - 1);
                            self.dirty.description = true;
                        }
                        Some(Due) => {
                            self.data.due.remove(current_index - 1);
                            self.dirty.due = true;
                        }
                        Some(PercentComplete) => {
                            self.data.percent_complete.remove(current_index - 1);
                            self.dirty.percent_complete = true;
                        }
                        Some(Priority) => {
                            self.data.priority.remove(current_index - 1);
                            self.dirty.priority = true;
                        }
                        Some(Status) => {
                            self.data.status.remove(current_index - 1);
                            self.dirty.status = true;
                        }
                        Some(Summary) => {
                            self.data.summary.remove(current_index - 1);
                            self.dirty.summary = true;
                        }
                        None => {}
                    }
                    self.character_index -= 1;
                }
            }
            KeyCode::Char(c) => {
                match self.fields.get(self.field_index) {
                    Some(Description) => {
                        self.data.description.insert(self.character_index, c);
                        self.dirty.description = true;
                    }
                    Some(Due) => {
                        self.data.due.insert(self.character_index, c);
                        self.dirty.due = true;
                    }
                    Some(PercentComplete) => {
                        self.data.percent_complete.insert(self.character_index, c);
                        self.dirty.percent_complete = true;
                    }
                    Some(Priority) => {
                        self.data.priority.insert(self.character_index, c);
                        self.dirty.priority = true;
                    }
                    Some(Status) => {
                        self.data.status.insert(self.character_index, c);
                        self.dirty.status = true;
                    }
                    Some(Summary) => {
                        self.data.summary.insert(self.character_index, c);
                        self.dirty.summary = true;
                    }
                    None => {}
                }
                self.character_index += 1;
            }
            _ => {}
        }
        None
    }

    fn new_with(data: Data) -> Self {
        let character_index = data.summary.len();
        Self {
            data,
            dirty: Dirty {
                description: false,
                due: false,
                percent_complete: false,
                priority: false,
                status: false,
                summary: false,
            },
            fields: vec![
                TodoEditorField::Summary,
                TodoEditorField::Due,
                TodoEditorField::PercentComplete,
                TodoEditorField::Priority,
                TodoEditorField::Status,
                TodoEditorField::Description,
            ],
            field_index: 0,
            character_index,
        }
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
                true => Some(ParsedPriority::from_str(&self.data.priority, IGNORE_CASE)?.into()),
                false => None,
            },
            status: match self.dirty.status {
                true => Some(ArgTodoStatus::from_str(&self.data.status, IGNORE_CASE)?.into()),
                false => None,
            },
            summary: self.dirty.summary.then(|| self.data.summary.clone()),
        })
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

        let areas = Layout::vertical(self.fields.iter().map(|_| Constraint::Max(3)))
            .margin(2)
            .split(area);

        for (i, (field, area)) in self.fields.iter().zip(areas.iter()).enumerate() {
            let content = match field {
                TodoEditorField::Description => &self.data.description,
                TodoEditorField::Due => &self.data.due,
                TodoEditorField::PercentComplete => &self.data.percent_complete,
                TodoEditorField::Priority => &self.data.priority,
                TodoEditorField::Status => &self.data.status,
                TodoEditorField::Summary => &self.data.summary,
            };
            let is_active = i == self.field_index;
            Paragraph::new(content.clone())
                .block(title_block(field.title(), is_active))
                .render(*area, buf);
        }
    }
}

/// Create a bordered block with a title.
fn title_block(title: &str, active: bool) -> Block {
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .padding(Padding::horizontal(1))
        .title(format!(" {title} ").bold());
    match active {
        true => block.blue(),
        false => block.gray(),
    }
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

#[derive(Debug, Default)]
struct Dirty {
    description: bool,
    due: bool,
    percent_complete: bool,
    priority: bool,
    status: bool,
    summary: bool,
}

#[derive(Debug, Clone, Copy)]
enum TodoEditorField {
    Description,
    Due,
    PercentComplete,
    Priority,
    Status,
    Summary,
}

impl TodoEditorField {
    const fn title(&self) -> &str {
        match self {
            TodoEditorField::Description => "Description",
            TodoEditorField::Due => "Due",
            TodoEditorField::PercentComplete => "Percent complete",
            TodoEditorField::Priority => "Priority",
            TodoEditorField::Status => "Status",
            TodoEditorField::Summary => "Summary",
        }
    }
}
