// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{self, Block, Paragraph, block};
use unicode_width::UnicodeWidthStr;

use crate::util::unicode_width_of_slice;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Handled,
    CursorUpdated,
    Submit,
    Exit,
}

pub trait Component<S> {
    /// Renders the component into the given area.
    fn render(&self, store: &S, area: Rect, buf: &mut Buffer);

    /// Handles key events for the component.
    fn on_key(&mut self, _store: &mut S, _area: Rect, _key: KeyCode) -> Option<Message> {
        None // Default implementation does nothing
    }

    /// Returns the cursor position (row, column) for the component, if applicable.
    fn get_cursor_position(&self, _store: &S, _area: Rect) -> Option<(u16, u16)> {
        None // Default implementation returns no cursor position
    }
}

pub trait FormItem<S>: Component<S> {
    fn activate(&mut self);
    fn deactivate(&mut self);
}

pub struct Form<S> {
    title: String,
    items: Vec<Box<dyn FormItem<S>>>,
    item_index: usize,
}

impl<S> Form<S> {
    pub fn new(title: String, items: Vec<Box<dyn FormItem<S>>>) -> Self {
        Self {
            title,
            items,
            item_index: 0,
        }
    }

    pub fn activate(&mut self) {
        if let Some(item) = self.items.get_mut(self.item_index) {
            item.activate();
        }
    }

    fn layout(&self) -> Layout {
        Layout::vertical(self.items.iter().map(|_| Constraint::Max(3))).margin(2)
    }
}

impl<S> Component<S> for Form<S> {
    fn render(&self, store: &S, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" {} ", self.title).bold());
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
        for (field, area) in self.items.iter().zip(areas.iter()) {
            field.render(store, *area, buf);
        }
    }

    fn on_key(&mut self, store: &mut S, area: Rect, key: KeyCode) -> Option<Message> {
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
                return Some(msg);
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
                if let Some(a) = self.items.get_mut(self.item_index) {
                    a.activate()
                }
                Some(Message::CursorUpdated)
            }
            KeyCode::Enter => Some(Message::Submit),
            _ => None,
        }
    }

    fn get_cursor_position(&self, store: &S, area: Rect) -> Option<(u16, u16)> {
        let areas = self.layout().split(area);
        match (self.items.get(self.item_index), areas.get(self.item_index)) {
            (Some(comp), Some(area)) => comp.get_cursor_position(store, *area),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Input<S> {
    title: String,
    active: bool,
    character_index: usize,
    get: fn(&S) -> &str,
    set: fn(&mut S, String),
    _marker: std::marker::PhantomData<S>,
}

impl<S> Input<S> {
    pub fn new(title: String, get: fn(&S) -> &str, set: fn(&mut S, String)) -> Self {
        Self {
            title,
            active: false,
            character_index: 0,
            get,
            set,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S> Component<S> for Input<S> {
    fn render(&self, store: &S, area: Rect, buf: &mut Buffer) {
        let block = title_block(&self.title, self.active);
        Paragraph::new((self.get)(store))
            .block(block)
            .render(area, buf);
    }

    fn on_key(&mut self, store: &mut S, _area: Rect, key: KeyCode) -> Option<Message> {
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
                if self.character_index < (self.get)(store).len() {
                    self.character_index += 1;
                }
            }
            KeyCode::Backspace => {
                if self.character_index > 0 {
                    let mut value = (self.get)(store).to_owned();
                    value.remove(self.character_index - 1);
                    (self.set)(store, value);
                    self.character_index -= 1;
                }
            }
            KeyCode::Char(c) => {
                let mut value = (self.get)(store).to_owned();
                value.insert(self.character_index, c);
                (self.set)(store, value);
                self.character_index += 1;
            }
            _ => return None,
        };

        // Always update the cursor position for simplicity
        Some(Message::CursorUpdated)
    }

    fn get_cursor_position(&self, store: &S, area: Rect) -> Option<(u16, u16)> {
        let width = unicode_width_of_slice((self.get)(store), self.character_index);
        let x = area.x + (width as u16) + 2; // border 1 + padding 1
        let y = area.y + 1; // title line: 1
        Some((x, y))
    }
}

impl<S> FormItem<S> for Input<S> {
    fn activate(&mut self) {
        self.active = true;
        self.character_index = 0; // Reset character index when activated
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.character_index = 0; // Reset character index when deactivated
    }
}

#[derive(Debug)]
pub struct RadioGroup<S, T: Eq> {
    title: String,
    values: Vec<T>,
    options: Vec<String>,
    get: fn(&S) -> T,
    set: fn(&mut S, &T),
    active: bool,
    _marker: std::marker::PhantomData<S>,
}

impl<S, T: Eq> RadioGroup<S, T> {
    pub fn new(
        title: String,
        values: Vec<T>,
        options: Vec<String>,
        get: fn(&S) -> T,
        set: fn(&mut S, &T),
    ) -> Self {
        Self {
            title,
            values,
            options,
            get,
            set,
            active: false,
            _marker: std::marker::PhantomData,
        }
    }

    fn selected(&self, store: &S) -> usize {
        let v = (self.get)(store);
        self.values.iter().position(|s| s == &v).unwrap_or(0)
    }

    fn split(&self, area: Rect) -> (Block, Rc<[Rect]>) {
        let outer_block = title_block(&self.title, self.active);
        let inner = outer_block.inner(area);
        let inner_blocks = self.layout().split(inner);
        (outer_block, inner_blocks)
    }

    fn layout(&self) -> Layout {
        let constraints = self
            .options
            .iter()
            // 6 = border left (1) + active marker [ ] (3) + space (1) + border right (1)
            .map(|s| Constraint::Min(6 + s.width() as u16));

        Layout::horizontal(constraints)
    }
}

impl<S, T: Eq> Component<S> for RadioGroup<S, T> {
    fn render(&self, store: &S, area: Rect, buf: &mut Buffer) {
        let (outer, inners) = self.split(area);
        outer.render(area, buf);
        for (i, (value, area)) in self.options.iter().zip(inners.iter()).enumerate() {
            let label = format!(
                "[{}] {}",
                if self.selected(store) == i { 'x' } else { ' ' },
                value
            );
            let inner_block = Block::default().padding(block::Padding::horizontal(1));
            Paragraph::new(label).block(inner_block).render(*area, buf);
        }
    }

    fn on_key(&mut self, store: &mut S, _area: Rect, key: KeyCode) -> Option<Message> {
        if !self.active {
            return None; // Only handle keys when the field is active
        }

        match key {
            KeyCode::Left | KeyCode::Right => {
                let offset = match key {
                    KeyCode::Left => self.values.len() - 1,
                    KeyCode::Right => 1,
                    _ => 0,
                };
                let index = (self.selected(store) + offset) % self.values.len();
                match self.values.get(index) {
                    Some(a) => {
                        (self.set)(store, a);
                        Some(Message::CursorUpdated)
                    }
                    None => Some(Message::Handled),
                }
            }
            _ => None,
        }
    }

    fn get_cursor_position(&self, store: &S, area: Rect) -> Option<(u16, u16)> {
        let (_, inners) = self.split(area);
        inners.get(self.selected(store)).map(|area| {
            let x = area.x + 2; // 2 = padding (1) + active marker [ (1)
            (x, area.y)
        })
    }
}

impl<S, T: Eq> FormItem<S> for RadioGroup<S, T> {
    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
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
