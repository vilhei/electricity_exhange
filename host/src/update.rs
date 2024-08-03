use color_eyre::eyre::Context;
use tracing::{event, info, instrument, trace, Level};

use crate::model::{Model, RunningState};

pub fn update(model: &mut Model, message: &UiMessage) -> Option<UiMessage> {
    match message {
        UiMessage::FetchSerialPorts => update_serial_ports(model),
        UiMessage::ForceQuit => force_quit(model),
        UiMessage::SelectionUp => move_selection_up(model),
        UiMessage::SelectionDown => move_selection_down(model),
        UiMessage::ClearSelection => clear_selection(model),
        UiMessage::SelectLast => move_selection_to_last(model),
        UiMessage::SelectFirst => move_selection_to_first(model),
    }
}

#[instrument]
fn update_serial_ports(model: &mut Model) -> Option<UiMessage> {
    if let RunningState::SelectSerialPort(s) = &mut model.running_state {
        // trace!("Fetching serial ports");
        trace!("Fetching serial ports");
        s.ports = serialport::available_ports()
            .wrap_err("Failed to fetch serial ports")
            .unwrap();
    } else {
        panic!(
            "Cannot fetch serial ports if not in SelectSerialPort state, currently in {:?}",
            model.running_state
        );
    }

    None
}

fn force_quit(model: &mut Model) -> Option<UiMessage> {
    model.running_state = RunningState::ForceQuit;
    None
}

fn move_selection_up(model: &mut Model) -> Option<UiMessage> {
    trace!("selection up");
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => {
            if let Some(0) = state.list_state.selected() {
                return Some(UiMessage::SelectLast);
            }

            state.list_state.select_previous();
        }
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}
fn move_selection_down(model: &mut Model) -> Option<UiMessage> {
    trace!("selection down");
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => {
            if let Some(idx) = state.list_state.selected() {
                if idx == state.ports.len() - 1 {
                    return Some(UiMessage::SelectFirst);
                }
            }
            state.list_state.select_next();
        }
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

fn move_selection_to_last(model: &mut Model) -> Option<UiMessage> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => state.list_state.select_last(),
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

fn move_selection_to_first(model: &mut Model) -> Option<UiMessage> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => state.list_state.select_first(),
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

fn clear_selection(model: &mut Model) -> Option<UiMessage> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => state.list_state.select(None),
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }
    None
}

pub enum UiMessage {
    FetchSerialPorts,
    /// Quits the application without saving anything or checking with user
    ForceQuit,
    SelectionUp,
    SelectionDown,
    ClearSelection,
    SelectLast,
    SelectFirst,
}
