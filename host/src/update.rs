use color_eyre::eyre::Context;
use host::{action::Action, title_block};
use ratatui::widgets::ListState;
use strum::{EnumCount, VariantNames};
use tracing::{info, instrument, trace, warn, Level};

use crate::model::{MainScreenState, Model, PopUpState, RunningState, SerialPortScreenState};

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
        Action::SerialPortConnectionFail => serial_connection_failed(model),
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
            "Cannot fetch serial ports if not in SelectSerialPort state, currently in {}",
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
            return select_previous_wrapping(&mut state.list_state)
        }
        RunningState::Main(state) => return select_previous_wrapping(&mut state.list_state),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_down(model: &mut Model) -> Option<Action> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => {
            return select_next_wrapping(&mut state.list_state, state.ports.len() - 1)
        }
        RunningState::Main(state) => {
            return select_next_wrapping(&mut state.list_state, shared::Message::COUNT - 1)
        }

        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }
}

fn select_previous_wrapping(list_state: &mut ListState) -> Option<Action> {
    if let Some(0) = list_state.selected() {
        return Some(Action::SelectLast);
    }
    list_state.select_previous();
    None
}

fn select_next_wrapping(list_state: &mut ListState, max: usize) -> Option<Action> {
    if let Some(idx) = list_state.selected() {
        if idx == max {
            return Some(Action::SelectFirst);
        }
    }
    list_state.select_next();
    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_to_last(model: &mut Model) -> Option<Action> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => state.list_state.select_last(),
        RunningState::Main(state) => state.list_state.select_last(),
        RunningState::Configure(_) => todo!(),
        RunningState::GetInformation(_) => todo!(),
        RunningState::Quit(_) => todo!(),
        RunningState::ForceQuit => todo!(),
    }

    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn move_selection_to_first(model: &mut Model) -> Option<Action> {
    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => state.list_state.select_first(),
        RunningState::Main(state) => state.list_state.select_first(),
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
                let selected_port: &str = &state.ports[idx].port_name;
                info!(target:"state_change","from serialport to main with port {}",selected_port);
                let serial_port = serialport::new(selected_port, 115200)
                    .timeout(std::time::Duration::from_millis(50))
                    .open();

                let mut serial_port = match serial_port {
                    Ok(s) => s,
                    Err(_) => return Some(Action::SerialPortConnectionFail),
                };

                serial_port
                    .set_data_bits(serialport::DataBits::Eight)
                    .unwrap();
                serial_port
                    .set_stop_bits(serialport::StopBits::One)
                    .unwrap();
                serial_port.set_parity(serialport::Parity::None).unwrap();

                model.running_state =
                    RunningState::Main(MainScreenState::with_serial_port(serial_port));
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
            model.popup = Some(PopUpState::Message(
                "Select a serial port to continue".to_string(),
            ))
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
fn serial_connection_failed(model: &mut Model) -> Option<Action> {
    if let RunningState::SelectSerialPort(s) = &model.running_state {
        // Port should always be selected because continuing from serial port to main state without port is not allowed
        let idx = s
            .list_state
            .selected()
            .expect("No port selected when failed connection should not be possible");
        let selected_port = &s.ports[idx].port_name;

        warn!("Failed to connect to serial port {}", selected_port);

        model.popup = Some(PopUpState::Message(format!(
            "Failed to connect to selected serial port : {}\nMake sure you selected the right port",
            selected_port
        )));
    } else {
        panic!(
            "serial_connection_failed called in illegal state : {}",
            model.running_state
        )
    }
    None
}

#[instrument(ret(level=Level::TRACE), skip_all, fields(state = model.running_state.to_string()))]
fn close_popup(model: &mut Model) -> Option<Action> {
    model.popup = None;
    None
}
