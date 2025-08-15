// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use ratatui::crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::tui::component::{Component, Message};
use crate::tui::dispatcher::{Action, Dispatcher, EventOrTodo};

pub struct SinglePage<S, C: Component<S>> {
    title: String,
    inner: C,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, C: Component<S>> SinglePage<S, C> {
    pub fn new(title: String, inner: C) -> Self {
        Self {
            title,
            inner,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, C: Component<S>> Component<S> for SinglePage<S, C> {
    fn render(&self, store: &Rc<RefCell<S>>, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" {} ", self.title).bold());
        let block = block()
            .title(title.centered())
            .title_bottom(instructions().centered())
            .white();

        let inner_area = block.inner(area);
        block.render(area, buf);
        self.inner.render(store, inner_area, buf);
    }

    fn get_cursor_position(&self, store: &Rc<RefCell<S>>, area: Rect) -> Option<(u16, u16)> {
        let inner_area = block().inner(area);
        self.inner.get_cursor_position(store, inner_area)
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Rc<RefCell<S>>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        if let Some(msg) = self.inner.on_key(dispatcher, store, area, key) {
            return Some(msg);
        }

        match key {
            KeyCode::Esc => Some(Message::Exit),
            _ => None,
        }
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher) {
        self.inner.activate(dispatcher);
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher) {
        self.inner.deactivate(dispatcher);
    }
}

pub struct TabPages<S> {
    identifiers: Vec<EventOrTodo>,
    titles: Vec<String>,
    pages: Vec<Box<dyn Component<S>>>,
    active_index: usize,
    active: bool,
    tab_active: bool,
}

impl<S> TabPages<S> {
    pub fn new(pages: Vec<(EventOrTodo, String, Box<dyn Component<S>>)>) -> Self {
        let len = pages.len();
        let (identifiers, titles, pages) = pages.into_iter().fold(
            (
                Vec::with_capacity(len),
                Vec::with_capacity(len),
                Vec::with_capacity(len),
            ),
            |(mut v1, mut v2, mut v3), (a, b, c)| {
                v1.push(a);
                v2.push(b);
                v3.push(c);
                (v1, v2, v3)
            },
        );

        Self {
            identifiers,
            titles,
            pages,
            active_index: 0,
            active: false,
            tab_active: true,
        }
    }

    fn layout(&self) -> Layout {
        Layout::vertical([Constraint::Max(1), Constraint::Fill(1)])
    }
}

impl<S> Component<S> for TabPages<S> {
    fn render(&self, store: &Rc<RefCell<S>>, area: Rect, buf: &mut Buffer) {
        let areas = self.layout().split(area);

        if let Some(area) = areas.get(0) {
            let tabs = Layout::horizontal(
                self.titles
                    .iter()
                    .map(|title| Constraint::Min(2 + title.width() as u16)),
            );

            let areas = tabs.split(*area);
            for (i, (title, area)) in self.titles.iter().zip(areas.iter()).enumerate() {
                const STYLE_SELECTED: Style = Style::new().fg(Color::Black).bg(Color::White);
                const STYLE_ACTIVE: Style = Style::new().fg(Color::Black).bg(Color::Blue);
                const STYLE_INACTIVE: Style = Style::new().fg(Color::White);

                let style = if !self.active || i != self.active_index {
                    STYLE_INACTIVE
                } else if self.tab_active {
                    STYLE_ACTIVE
                } else {
                    STYLE_SELECTED
                };

                Paragraph::new(title.as_str())
                    .style(style)
                    .centered()
                    .render(*area, buf);
            }
        }

        if let Some(area) = areas.get(1)
            && let Some(active_page) = self.pages.get(self.active_index)
        {
            let block = block();
            active_page.render(store, block.inner(*area), buf);
            block.render(*area, buf);
        }
    }

    fn get_cursor_position(&self, store: &Rc<RefCell<S>>, area: Rect) -> Option<(u16, u16)> {
        if !self.active || self.tab_active {
            return None; // No cursor in tab bar
        }

        let areas = self.layout().split(area);
        if let Some(active_page) = self.pages.get(self.active_index)
            && let Some(area) = areas.get(1)
        {
            let inner_area = block().inner(*area);
            active_page.get_cursor_position(store, inner_area)
        } else {
            None
        }
    }

    fn on_key(
        &mut self,
        dispatcher: &mut Dispatcher,
        store: &Rc<RefCell<S>>,
        area: Rect,
        key: KeyCode,
    ) -> Option<Message> {
        let len = self.pages.len();
        if self.tab_active {
            match key {
                KeyCode::Down => {
                    self.tab_active = false;
                    if let Some(page) = self.pages.get_mut(self.active_index) {
                        page.activate(dispatcher);
                    }
                    Some(Message::CursorUpdated)
                }
                KeyCode::Left if self.tab_active && self.active_index > 0 => {
                    self.active_index -= 1;
                    if let Some(id) = self.identifiers.get_mut(self.active_index) {
                        dispatcher.dispatch(Action::Activate(id.clone()));
                    }
                    Some(Message::Handled)
                }
                KeyCode::Right if self.tab_active && self.active_index < len - 1 => {
                    self.active_index += 1;
                    if let Some(id) = self.identifiers.get_mut(self.active_index) {
                        dispatcher.dispatch(Action::Activate(id.clone()));
                    }
                    Some(Message::Handled)
                }
                KeyCode::Esc => Some(Message::Exit),
                _ => None,
            }
        } else if let Some(page) = self.pages.get_mut(self.active_index) {
            match page.on_key(dispatcher, store, area, key) {
                Some(msg) => Some(msg),
                None => match key {
                    KeyCode::Up if !self.tab_active => {
                        self.tab_active = true;
                        page.deactivate(dispatcher);
                        Some(Message::CursorUpdated)
                    }
                    KeyCode::Esc => Some(Message::Exit),
                    _ => None,
                },
            }
        } else {
            None
        }
    }

    fn activate(&mut self, dispatcher: &mut Dispatcher) {
        self.active = true;
        if !self.tab_active
            && let Some(active_page) = self.pages.get_mut(self.active_index)
        {
            active_page.activate(dispatcher);
        }
    }

    fn deactivate(&mut self, dispatcher: &mut Dispatcher) {
        self.active = false;
        if !self.tab_active
            && let Some(active_page) = self.pages.get_mut(self.active_index)
        {
            active_page.deactivate(dispatcher);
        }
    }
}

fn instructions() -> Line<'static> {
    Line::from(vec![
        " Prev ".into(),
        "<Up>".blue().bold(),
        " Next ".into(),
        "<Down>".blue().bold(),
        " Exit ".into(),
        "<Esc> ".blue().bold(),
    ])
}

fn block() -> Block<'static> {
    Block::bordered().border_set(border::ROUNDED)
}
