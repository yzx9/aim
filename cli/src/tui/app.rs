// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_core::{Aim, Todo, TodoDraft, TodoPatch};

use crate::tui::{component::Message, todo_editor::TodoEditor, todo_store::TodoStore};

pub fn draft_todo(aim: &mut Aim) -> Result<Option<TodoDraft>, Box<dyn Error>> {
    let draft = aim.default_todo_draft();
    let mut store = TodoStore::new_by_draft(draft);
    match run_todo_editor(aim, &mut store)? {
        true => store.submit_draft().map(Some),
        false => Ok(None),
    }
}

pub fn patch_todo(aim: &mut Aim, todo: &impl Todo) -> Result<Option<TodoPatch>, Box<dyn Error>> {
    let mut store = TodoStore::new_by_todo(todo);
    match run_todo_editor(aim, &mut store)? {
        true => store.submit_patch().map(Some),
        false => Ok(None),
    }
}

fn run_todo_editor(aim: &mut Aim, store: &mut TodoStore) -> Result<bool, Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut view = TodoEditor::new(store, &mut terminal);
    let result = loop {
        if let Err(e) = view.darw(store, &mut terminal) {
            break Err(e);
        }

        match view.read_event(store) {
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
