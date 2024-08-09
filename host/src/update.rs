use color_eyre::eyre::Context;
use host::action::Action;
use tracing::{info, instrument, trace, Level};

use crate::model::{MainScreenState, Model, PopUpState, RunningState};

pub fn update(model: &mut Model, message: &Action) -> Option<Action> {
    match message {
        Action::FetchSerialPorts => fetch_serial_ports(model),
        Action::ForceQuit => force_quit(model),
        Action::SelectionUp => move_selection_up(model),
        Action::SelectionDown => move_selection_down(model),
        Action::ClearSelection => clear_selection(model),
        Action::SelectLast => move_selection_to_last(model),
        Action::SelectFirst => move_selection_to_first(model),
        Action::StateChangeFromSerialPortToMain => move_from_serialport_to_main(model),
        Action::MustSelectOne => must_select_one(model),
        Action::ClosePopUp => close_popup(model),
        Action::ShowKeyBindings => show_keybindings(model),
    }
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state=model.running_state.to_string(), ports_fetched))]
fn fetch_serial_ports(model: &mut Model) -> Option<Action> {
    if let RunningState::SelectSerialPort(s) = &mut model.running_state {
        // trace!("Fetching serial ports");
        // trace!("Fetching serial ports");
        s.ports = serialport::available_ports()
            .wrap_err("Failed to fetch serial ports")
            .unwrap();
        tracing::Span::current().record("ports_fetched", s.ports.len());
    } else {
        panic!(
            "Cannot fetch serial ports if not in SelectSerialPort state, currently in {:?}",
            model.running_state
        );
    }
    None
}

fn force_quit(model: &mut Model) -> Option<Action> {
    info!("Quitting");
    model.running_state = RunningState::ForceQuit;
    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state=model.running_state.to_string()))]
fn move_selection_up(model: &mut Model) -> Option<Action> {
    // trace!("selection up");
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => {
            if let Some(0) = state.list_state.selected() {
                return Some(Action::SelectLast);
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

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_down(model: &mut Model) -> Option<Action> {
    // trace!("selection down");
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => {
            if let Some(idx) = state.list_state.selected() {
                if idx == state.ports.len() - 1 {
                    return Some(Action::SelectFirst);
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

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_to_last(model: &mut Model) -> Option<Action> {
    info!("selection last");
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

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_to_first(model: &mut Model) -> Option<Action> {
    trace!("selection first");
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

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn clear_selection(model: &mut Model) -> Option<Action> {
    trace!("clear selection");
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

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_from_serialport_to_main(model: &mut Model) -> Option<Action> {
    if let RunningState::SelectSerialPort(state) = &mut model.running_state {
        match state.list_state.selected() {
            Some(idx) => {
                let selected_port = state.ports.swap_remove(idx).port_name;
                info!(target:"state_change","from serialport to main with port {}",selected_port);

                model.running_state = RunningState::Main(MainScreenState {
                    port_name: selected_port,
                });
            }
            None => return Some(Action::MustSelectOne),
        };
    } else {
        panic!(
            "Illegal action StateChangeFromSerialPortToMain in state : {}",
            model.running_state
        );
    }
    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn must_select_one(model: &mut Model) -> Option<Action> {
    match model.running_state {
        RunningState::SelectSerialPort(_) => {
            model.popup = Some(PopUpState::Message("Select a serial port to continue"))
        }
        RunningState::Main(_) => todo!(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

fn show_keybindings(model: &mut Model) -> Option<Action> {
    model.popup = Some(PopUpState::ShowKeyBindings);
    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn close_popup(model: &mut Model) -> Option<Action> {
    model.popup = None;
    None
}
