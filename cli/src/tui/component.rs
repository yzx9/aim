// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{self, Block, Paragraph, block};
use unicode_width::UnicodeWidthStr;

use crate::util::unicode_width_of_slice;

#[derive(Debug)]
pub struct Input<'a> {
    pub title: &'a str,
    pub value: &'a str,
    pub active: bool,
}

impl<'a> Input<'a> {
    pub fn get_cursor_position(&self, area: Rect, character_index: usize) -> (u16, u16) {
        let width = unicode_width_of_slice(self.value, character_index);
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

#[derive(Debug)]
pub struct Switch<'a> {
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
