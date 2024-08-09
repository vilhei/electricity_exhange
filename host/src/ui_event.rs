use std::time::Duration;

use color_eyre::{eyre::Ok, Result};
use host::action::Action;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent};

use crate::model::Model;

pub fn handle_event(model: &Model) -> Result<Option<Action>> {
    if event::poll(Duration::from_millis(50))? {
        match &event::read()? {
            event::Event::Key(k) if k.kind == event::KeyEventKind::Press => {
                handle_key_event(k, model)
            }
            event::Event::Key(_) => Ok(None),
            event::Event::FocusGained => Ok(None),
            event::Event::FocusLost => Ok(None),
            event::Event::Mouse(_) => Ok(None),
            event::Event::Paste(_) => Ok(None),
            event::Event::Resize(_, _) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn handle_key_event(key_event: &KeyEvent, model: &Model) -> Result<Option<Action>> {
    let possible_action = model.get_keybinding().get(key_event);
    // Handle force quit before other key events
    if possible_action.is_some_and(|a| matches!(a, Action::ForceQuit)) {
        return Ok(Some(Action::ForceQuit));
    }

    // Force user to close possible popup before continuing with other actions
    if model.popup.is_some() {
        match key_event.code {
            KeyCode::Esc => return Ok(Some(Action::ClosePopUp)),
            _ => return Ok(None),
        }
    }
    Ok(possible_action.copied())
}

#[allow(unused)]
pub fn handle_select_serial_port_key_event(
    key_event: &KeyEvent,
    model: &Model,
) -> Result<Option<Action>> {
    Ok(model.get_keybinding().get(key_event).copied())
}
#[allow(unused)]
pub fn handle_main_key_event(key_event: &KeyEvent) -> Result<Option<Action>> {
    todo!()
}

#[allow(unused)]
pub fn handle_configure_key_event(key_event: &KeyEvent) -> Result<Option<Action>> {
    todo!()
}
#[allow(unused)]
pub fn handle_get_information_key_event(key_event: &KeyEvent) -> Result<Option<Action>> {
    todo!()
}
#[allow(unused)]
pub fn handle_quit_key_event(key_event: &KeyEvent) -> Result<Option<Action>> {
    todo!()
}
