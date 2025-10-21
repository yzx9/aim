// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::{Action, Dispatcher};
use crate::util::{byte_range_of_grapheme_at, unicode_width_of_slice};

pub struct Form<S, C: FormItem<S>> {
    items: Vec<C>,
    item_index: usize,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, C: FormItem<S>> Form<S, C> {
    pub fn new(items: Vec<C>) -> Self {
        Self {
            items,
            item_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn layout(&self, store: &RefCell<S>) -> Layout {
        Layout::vertical(self.items.iter().map(|item| match item.item_state(store) {
            FormItemState::Invisible => Constraint::Max(0),
            _ => Constraint::Max(3),
        }))
        .margin(1)
    }

    fn navigate(&mut self, dispatcher: &mut Dispatcher, store: &RefCell<S>, offset: isize) {
        // deactivate current item
        if let Some(a) = self.items.get_mut(self.item_index) {
            a.deactivate(dispatcher, store);
        }

        // move to next/previous item, skipping invisible items
        let len = self.items.len();

        // find the next visible item
        let mut new_index = self.item_index;
        let mut steps = offset.unsigned_abs();

        while steps > 0 {
            if offset > 0 {
                new_index = (new_index + 1) % len;
            } else {
                new_index = (new_index + len - 1) % len;
            }

            // Check if the item at new_index is visible
            if let Some(item) = self.items.get(new_index)
                && item_is_visible(item, store)
            {
                steps -= 1; // Found a visible item
            } else if new_index == self.item_index {
                // If we've gone through all items and none are visible, break
                break;
            }
        }

        self.item_index = new_index;

        // activate new item
        if let Some(a) = self.items.get_mut(self.item_index) {
            a.activate(dispatcher, store);
        }
    }
}

impl<S, C: FormItem<S>> Component<S> for Form<S, C> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        let areas = self.layout(store).split(area);
        let mut is_last = true;
        for (item, area) in self.items.iter().zip(areas.iter()).rev() {
            // reverse order to draw the last item first, dont assert if the item is visible
            if item_is_visible(item, store) {
                item_render(is_last, item, store, *area, buf);
                item.render(store, item_inner(*area), buf);
                is_last = false;
            }
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        self.items
            .iter()
            .zip(self.layout(store).split(area).iter())
            .take(self.item_index + 1)
            .last()
            .and_then(|(comp, area)| comp.get_cursor_position(store, *area))
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        // Handle key events for the current component
        let areas = self.layout(store).split(area);
        if let Some((comp, subarea)) = self
            .items
            .iter_mut()
            .zip(areas.iter())
            .take(self.item_index + 1)
            .last()
            && let Some(msg) = comp.on_key(dispatcher, store, *subarea, event)
        {
            return Some(msg);
        };

        match event.code {
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

pub trait FormItem<S>: Component<S> {
    fn item_title(&self, store: &RefCell<S>) -> &str;
    fn item_state(&self, store: &RefCell<S>) -> FormItemState;
}

impl<S> FormItem<S> for Box<dyn FormItem<S>> {
    fn item_title(&self, store: &RefCell<S>) -> &str {
        (**self).item_title(store)
    }

    fn item_state(&self, store: &RefCell<S>) -> FormItemState {
        (**self).item_state(store)
    }
}

pub enum FormItemState {
    // Whether the component is currently active (focused).
    Active,

    // Whether the component is currently inactive (not focused).
    Inactive,

    /// Whether the component is currently visible. By default, all items are visible.
    Invisible,
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
    pub fn new(title: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            active: false,
            character_index: 0,
            _phantom_a: std::marker::PhantomData,
            _phantom_s: std::marker::PhantomData,
        }
    }
}

impl<S, A: Access<S, String>> Component<S> for Input<S, A> {
    fn render(&self, store: &RefCell<S>, area: Rect, buf: &mut Buffer) {
        let v = A::get(store);
        Paragraph::new(v.as_str()).render(area, buf);
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
        event: KeyEvent,
    ) -> Option<Message> {
        use KeyCode::*;
        if !self.active || !matches!(event.code, Left | Right | Backspace | Char(_)) {
            return None;
        }

        match event.code {
            Left if self.character_index > 0 => self.character_index -= 1,
            Right if self.character_index < A::get(store).len() => self.character_index += 1,
            Backspace if self.character_index > 0 => {
                let mut v = A::get(store);
                if let Some(range) = byte_range_of_grapheme_at(&v, self.character_index - 1) {
                    v.replace_range(range, "");
                    if A::set(dispatcher, v) {
                        self.character_index -= 1;
                    }
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

impl<S, A: Access<S, String>> FormItem<S> for Input<S, A> {
    fn item_title(&self, _store: &RefCell<S>) -> &str {
        &self.title
    }

    fn item_state(&self, _store: &RefCell<S>) -> FormItemState {
        if self.active {
            FormItemState::Active
        } else {
            FormItemState::Inactive
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
    pub fn new(title: impl ToString, values: Vec<T>, options: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
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

    fn split(&self, area: Rect) -> Rc<[Rect]> {
        self.layout().split(area)
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
        let options = self.split(area);
        for (i, (value, area)) in self.options.iter().zip(options.iter()).enumerate() {
            let icon = if self.selected(store) == i { 'x' } else { ' ' };
            let label = format!("[{icon}] {value}");
            Paragraph::new(label).render(*area, buf);
        }
    }

    fn get_cursor_position(&self, store: &RefCell<S>, area: Rect) -> Option<(u16, u16)> {
        self.split(item_inner(area))
            .get(self.selected(store))
            .map(|area| (area.x + 1, area.y))
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &RefCell<S>,
        _area: Rect,
        event: KeyEvent,
    ) -> Option<Message> {
        if !self.active {
            return None; // Only handle keys when the field is active
        }

        match event.code {
            KeyCode::Left | KeyCode::Right => {
                let offset = match event.code {
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

impl<S, T: Eq + Clone, A: Access<S, T>> FormItem<S> for RadioGroup<S, T, A> {
    fn item_title(&self, _store: &RefCell<S>) -> &str {
        &self.title
    }

    fn item_state(&self, _store: &RefCell<S>) -> FormItemState {
        if self.active {
            FormItemState::Active
        } else {
            FormItemState::Inactive
        }
    }
}

const S_STEP_ACTIVE: &str = "◆";
const S_STEP_INACTIVE: &str = "◇";
// const S_STEP_CANCEL: &str = "■";
// const S_STEP_ERROR: &str = "▲";

// const S_SIDER_TOP: &str = "┌";
const S_SIDER_CONNECTOR: &str = "│";
const S_SIDER_BOTTOM: &str = "└";

fn item_render<S>(
    is_last: bool,
    item: &impl FormItem<S>,
    store: &RefCell<S>,
    area: Rect,
    buf: &mut Buffer,
) {
    let color = match item.item_state(store) {
        FormItemState::Active => Color::Blue,
        FormItemState::Inactive => Color::Gray,
        FormItemState::Invisible => return,
    };

    let area_title = Rect::new(area.x + 2, area.y, area.width.saturating_sub(2), 1);
    Clear.render(area_title, buf);
    Paragraph::new(item.item_title(store))
        .bold()
        .fg(color)
        .render(area_title, buf);

    if let Some(c) = buf.cell_mut((area.x, area.y)) {
        let symbol = match item.item_state(store) {
            FormItemState::Active => S_STEP_ACTIVE,
            FormItemState::Inactive => S_STEP_INACTIVE,
            FormItemState::Invisible => unreachable!(),
        };
        c.set_symbol(symbol);
        c.set_fg(color);
    }

    for y in 1..area.height.saturating_sub(1) {
        if let Some(c) = buf.cell_mut((area.x, area.y + y)) {
            c.set_symbol(S_SIDER_CONNECTOR);
            c.set_fg(color);
        }
    }

    if let Some(c) = buf.cell_mut((area.x, area.y + area.height.saturating_sub(1))) {
        let symbol = if is_last {
            S_SIDER_BOTTOM
        } else {
            S_SIDER_CONNECTOR
        };
        c.set_symbol(symbol);
        c.set_fg(color);
    }
}

fn item_inner(area: Rect) -> Rect {
    Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn item_is_visible<S>(item: &impl FormItem<S>, store: &RefCell<S>) -> bool {
    !matches!(item.item_state(store), FormItemState::Invisible)
}
