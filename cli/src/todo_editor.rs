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
    active_field: usize,
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
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

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
                self.active_field = (self.active_field + len - 1) % len;
            }
            KeyCode::Down | KeyCode::Tab => {
                self.active_field = (self.active_field + 1) % self.fields.len();
            }
            KeyCode::Backspace => match self.fields.get(self.active_field) {
                Some(Description) => {
                    self.data.description.pop();
                    self.dirty.description = true;
                }
                Some(Due) => {
                    self.data.due.pop();
                    self.dirty.due = true;
                }
                Some(PercentComplete) => {
                    self.data.percent_complete.pop();
                    self.dirty.percent_complete = true;
                }
                Some(Priority) => {
                    self.data.priority.pop();
                    self.dirty.priority = true;
                }
                Some(Status) => {
                    self.data.status.pop();
                    self.dirty.status = true;
                }
                Some(Summary) => {
                    self.data.summary.pop();
                    self.dirty.summary = true;
                }
                None => {}
            },
            KeyCode::Char(c) => match self.fields.get(self.active_field) {
                Some(Description) => {
                    self.data.description.push(c);
                    self.dirty.description = true;
                }
                Some(Due) => {
                    self.data.due.push(c);
                    self.dirty.due = true;
                }
                Some(PercentComplete) => {
                    self.data.percent_complete.push(c);
                    self.dirty.percent_complete = true;
                }
                Some(Priority) => {
                    self.data.priority.push(c);
                    self.dirty.priority = true;
                }
                Some(Status) => {
                    self.data.status.push(c);
                    self.dirty.status = true;
                }
                Some(Summary) => {
                    self.data.summary.push(c);
                    self.dirty.summary = true;
                }
                None => {}
            },
            _ => {}
        }
        None
    }

    fn new_with(data: Data) -> Self {
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
            active_field: 0,
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
        let title = Line::from("Todo Editor".bold());
        let instructions = Line::from(vec![
            "  Prev ".into(),
            "<Up>".blue().bold(),
            " Next ".into(),
            "<Down>".blue().bold(),
            " Exit ".into(),
            "<Esc>  ".blue().bold(),
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
            let is_active = i == self.active_field;
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
