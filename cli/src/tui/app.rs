// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, error::Error, rc::Rc};

use aimcal_core::{Aim, Todo, TodoDraft, TodoPatch};

use crate::tui::component::Message;
use crate::tui::dispatcher::Dispatcher;
use crate::tui::todo_editor::TodoEditor;
use crate::tui::todo_store::TodoStore;

pub fn draft_todo(aim: &mut Aim) -> Result<Option<TodoDraft>, Box<dyn Error>> {
    let draft = aim.default_todo_draft();
    let store = TodoStore::new_by_draft(draft);
    let store = run_todo_editor(aim, store)?;
    match store.submit {
        true => store.submit_draft().map(Some),
        false => Ok(None),
    }
}

pub fn patch_todo(aim: &mut Aim, todo: &impl Todo) -> Result<Option<TodoPatch>, Box<dyn Error>> {
    let store = TodoStore::new_by_todo(todo);
    let store = run_todo_editor(aim, store)?;
    match store.submit {
        true => store.submit_patch().map(Some),
        false => Ok(None),
    }
}

fn run_todo_editor(aim: &mut Aim, store: TodoStore) -> Result<TodoStore, Box<dyn Error>> {
    let store = Rc::new(RefCell::new(store));

    let mut terminal = ratatui::init();
    let result = {
        let mut dispatcher = Dispatcher::new();
        TodoStore::register_to(store.clone(), &mut dispatcher);
        let mut view = TodoEditor::new(dispatcher, &store, &mut terminal);

        loop {
            if let Err(e) = view.darw(&store, &mut terminal) {
                break Err(e);
            }

            match view.read_event(&store) {
                Err(e) => break Err(e),
                Ok(Some(Message::Exit)) => break Ok(()),
                Ok(_) => {} // Continue the loop to render the next frame
            }
        }
    }; // release dispatcher and view here to avoid borrow conflicts
    ratatui::restore();
    aim.refresh_now(); // Ensure the current time is updated
    result?;

    let owned_store = Rc::try_unwrap(store)
        .map_err(|_| "Store still has references")?
        .into_inner();
    Ok(owned_store)
}
