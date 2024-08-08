use std::time::Duration;

use color_eyre::{eyre::Ok, Result};
use host::action::UiMessage;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};

use crate::model::Model;

pub fn handle_event(model: &Model) -> Result<Option<UiMessage>> {
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

fn handle_key_event(key_event: &KeyEvent, model: &Model) -> Result<Option<UiMessage>> {
    // Handle ctrl+c force quite before other key events
    if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c') {
        return Ok(Some(UiMessage::ForceQuit));
    }

    // Force user to close possible popup before continuing with other actions
    if model.popup.is_some() {
        match key_event.code {
            KeyCode::Esc => return Ok(Some(UiMessage::ClosePopUp)),
            _ => return Ok(None),
        }
    }

    match model.running_state {
        crate::model::RunningState::SelectSerialPort(_) => {
            handle_select_serial_port_key_event(key_event)
        }
        crate::model::RunningState::Main(_) => todo!(),
        crate::model::RunningState::Configure(_) => todo!(),
        crate::model::RunningState::GetInformation(_) => todo!(),
        crate::model::RunningState::Quit(_) => todo!(),
        crate::model::RunningState::ForceQuit => todo!(),
    }
}

pub fn handle_select_serial_port_key_event(key_event: &KeyEvent) -> Result<Option<UiMessage>> {
    match key_event.code {
        KeyCode::Char('f') => Ok(Some(UiMessage::FetchSerialPorts)),
        KeyCode::Up => Ok(Some(UiMessage::SelectionUp)),
        KeyCode::Down => Ok(Some(UiMessage::SelectionDown)),
        KeyCode::Esc => Ok(Some(UiMessage::ClearSelection)),
        KeyCode::Enter => Ok(Some(UiMessage::StateChangeFromSerialPortToMain)),
        _ => Ok(None),
    }
}
#[allow(unused)]
pub fn handle_main_key_event(key_event: &KeyEvent) -> Result<Option<UiMessage>> {
    todo!()
}

#[allow(unused)]
pub fn handle_configure_key_event(key_event: &KeyEvent) -> Result<Option<UiMessage>> {
    todo!()
}
#[allow(unused)]
pub fn handle_get_information_key_event(key_event: &KeyEvent) -> Result<Option<UiMessage>> {
    todo!()
}
#[allow(unused)]
pub fn handle_quit_key_event(key_event: &KeyEvent) -> Result<Option<UiMessage>> {
    todo!()
}
