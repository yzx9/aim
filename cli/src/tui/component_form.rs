// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc, str::FromStr};

use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{self, Block, Paragraph, block};
use unicode_width::UnicodeWidthStr;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::util::unicode_width_of_slice;

pub struct Form<S, C: Component<S>> {
    items: Vec<C>,
    item_index: usize,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, C: Component<S>> Form<S, C> {
    pub fn new(items: Vec<C>) -> Self {
        Self {
            items,
            item_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn layout(&self) -> Layout {
        Layout::vertical(self.items.iter().map(|_| Constraint::Max(3))).margin(1)
    }

    fn navigate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>, offset: isize) {
        // deactivate current item
        if let Some(a) = self.items.get_mut(self.item_index) {
            a.deactivate(dispatcher, store);
        }

        // move to next/previous item
        let len = self.items.len();
        let offset = if offset < 0 {
            let k = div_floor(offset, len as isize);
            (offset - k * (len as isize)) as usize
        } else {
            offset as usize
        };

        self.item_index = (self.item_index + len + offset) % len;

        // activate new item
        if let Some(a) = self.items.get_mut(self.item_index) {
            a.activate(dispatcher, store);
        }
    }
}

impl<S, C: Component<S>> Component<S> for Form<S, C> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        let areas = self.layout().split(area);
        for (field, area) in self.items.iter().zip(areas.iter()) {
            field.render(store, *area, buf);
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        let areas = self.layout().split(area);
        match (self.items.get(self.item_index), areas.get(self.item_index)) {
            (Some(comp), Some(area)) => comp.get_cursor_position(store, *area),
            _ => None,
        }
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        // Handle key events for the current component
        let areas = self.layout().split(area);
        if let Some((comp, subarea)) = self
            .items
            .iter_mut()
            .zip(areas.iter())
            .take(self.item_index + 1)
            .last()
            && let Some(msg) = comp.on_key(dispatcher, store, *subarea, key)
        {
            return Some(msg);
        };

        match key {
            KeyCode::Up | KeyCode::BackTab if self.item_index > 0 => {
                self.navigate(dispatcher, store, -1);
                Some(Message::CursorUpdated)
            }
            KeyCode::Down | KeyCode::Tab if self.item_index < self.items.len() - 1 => {
                self.navigate(dispatcher, store, 1);
                Some(Message::CursorUpdated)
            }
            KeyCode::Enter => {
                dispatcher.dispatch(Action::SubmitChanges);
                Some(Message::Exit)
            }
            _ => None,
        }
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        if let Some(item) = self.items.get_mut(self.item_index) {
            item.activate(dispatcher, store);
        }
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>) {
        if let Some(item) = self.items.get_mut(self.item_index) {
            item.deactivate(dispatcher, store);
        }
    }
}

pub trait Access<S, T: ToOwned> {
    fn get(store: &RefCell<S>) -> T;
    fn set(dispatcher: &mut Dispatcher, value: T) -> bool;
}

#[derive(Debug)]
pub struct Input<S, A: Access<S, String>> {
    title: String,
    active: bool,
    character_index: usize,
    _phantom_s: std::marker::PhantomData<S>,
    _phantom_a: std::marker::PhantomData<A>,
}

impl<S, A: Access<S, String>> Input<S, A> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            active: false,
            character_index: 0,
            _phantom_a: std::marker::PhantomData,
            _phantom_s: std::marker::PhantomData,
        }
    }
}

impl<S, A: Access<S, String>> Component<S> for Input<S, A> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        let block = form_item_block(&self.title, self.active);
        let v = A::get(store);
        Paragraph::new(v.as_str()).block(block).render(area, buf);
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        if !self.active {
            return None; // No cursor position when not active
        }

        let v = A::get(store);
        let width = unicode_width_of_slice(v.as_str(), self.character_index);
        let x = area.x + (width as u16) + 2; // border 1 + padding 1
        let y = area.y + 1; // title line: 1
        Some((x, y))
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        _area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        use KeyCode::*;
        if !self.active || !matches!(key, Left | Right | Backspace | Char(_)) {
            return None;
        }

        match key {
            Left if self.character_index > 0 => self.character_index -= 1,
            Right if self.character_index < A::get(store).len() => self.character_index += 1,
            Backspace if self.character_index > 0 => {
                let mut v = A::get(store);
                v.remove(self.character_index - 1);
                if A::set(dispatcher, v) {
                    self.character_index -= 1;
                }
            }
            Char(c) => {
                let mut v = A::get(store);
                let byte_index = v
                    .char_indices()
                    .nth(self.character_index)
                    .map(|(i, _)| i)
                    .unwrap_or(v.len());
                v.insert(byte_index, c);
                if A::set(dispatcher, v) {
                    self.character_index += 1;
                }
            }
            _ => {}
        };

        // Always update the cursor position for simplicity
        Some(Message::CursorUpdated)
    }

    fn activate(&mut self, _dispatcher: &mut Dispatcher, _store: &RefCell<S>) {
        self.active = true;
        self.character_index = 0; // Reset character index when activated
    }

    fn deactivate(&mut self, _dispatcher: &mut Dispatcher, _store: &RefCell<S>) {
        self.active = false;
        self.character_index = 0; // Reset character index when deactivated
    }
}

pub struct PositiveIntegerAccess<S, T, A>
where
    T: ToString + FromStr + ToOwned + Clone,
    A: Access<S, Option<T>>,
{
    _phantom_s: std::marker::PhantomData<S>,
    _phantom_a: std::marker::PhantomData<A>,
    _phantom_t: std::marker::PhantomData<T>,
}

impl<S, T, A> Access<S, String> for PositiveIntegerAccess<S, T, A>
where
    T: Eq + ToString + FromStr + ToOwned + Clone,
    A: Access<S, Option<T>>,
{
    fn get(s: &RefCell<S>) -> String {
        match A::get(s) {
            Some(a) => a.to_string(),
            None => "".to_string(),
        }
    }

    #[tracing::instrument(skip_all)]
    fn set(dispatcher: &mut Dispatcher, value: String) -> bool {
        let v = value.trim();
        if v.is_empty() {
            A::set(dispatcher, None)
        } else if let Ok(num) = v.parse::<T>() {
            A::set(dispatcher, Some(num))
        } else {
            tracing::debug!(value, "failed to parse as a positive integer");
            false
        }
    }
}

#[derive(Debug)]
pub struct RadioGroup<S, T: Eq + Clone, A: Access<S, T>> {
    title: String,
    values: Vec<T>,
    options: Vec<String>,
    active: bool,
    _phantom_s: std::marker::PhantomData<S>,
    _phantom_a: std::marker::PhantomData<A>,
}

impl<S, T: Eq + Clone, A: Access<S, T>> RadioGroup<S, T, A> {
    pub fn new(title: String, values: Vec<T>, options: Vec<String>) -> Self {
        Self {
            title,
            values,
            options,
            active: false,
            _phantom_s: std::marker::PhantomData,
            _phantom_a: std::marker::PhantomData,
        }
    }

    fn selected(&self, store: &RefCell<S>) -> usize {
        let v = A::get(store);
        self.values.iter().position(|s| s == &v).unwrap_or(0)
    }

    fn split(&self, area: Rect) -> (Block<'_>, Rc<[Rect]>) {
        let outer_block = form_item_block(&self.title, self.active);
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

impl<S, T: Eq + Clone, A: Access<S, T>> Component<S> for RadioGroup<S, T, A> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        let (outer, inners) = self.split(area);
        outer.render(area, buf);
        for (i, (value, area)) in self.options.iter().zip(inners.iter()).enumerate() {
            let icon = if self.selected(store) == i { 'x' } else { ' ' };
            let label = format!("[{icon}] {value}");
            let inner_block = Block::default().padding(block::Padding::horizontal(1));
            Paragraph::new(label).block(inner_block).render(*area, buf);
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        if !self.active {
            return None; // No cursor position when not active
        }

        let (_, inners) = self.split(area);
        inners.get(self.selected(store)).map(|area| {
            let x = area.x + 2; // 2 = padding (1) + active marker [ (1)
            (x, area.y)
        })
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        _area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
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
                        A::set(dispatcher, a.to_owned());
                        Some(Message::CursorUpdated)
                    }
                    None => Some(Message::Handled),
                }
            }
            _ => None,
        }
    }

    fn activate(&mut self, _: &mut Dispatcher, _store: &RefCell<S>) {
        self.active = true;
    }

    fn deactivate(&mut self, _: &mut Dispatcher, _store: &RefCell<S>) {
        self.active = false;
    }
}

fn form_item_block(title: &str, active: bool) -> Block<'_> {
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

fn div_floor(a: isize, b: isize) -> isize {
    let d = a / b; // Truncated division toward zero
    let r = a % b;
    if (r != 0) && ((r < 0) != (b < 0)) {
        d - 1
    } else {
        d
    }
}
