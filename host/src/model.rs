use std::io::{Read, Write};

use host::settings::{keybindings::KeyBindings, Settings};
use ratatui::widgets::ListState;
use serialport::{SerialPort, SerialPortInfo};

// #[derive(Debug)]
pub struct Model {
    pub running_state: RunningState,
    pub popup: Option<PopUpState>,
    pub settings: Settings,
}

impl Model {
    pub fn new() -> Self {
        Self {
            running_state: RunningState::SelectSerialPort(Default::default()),
            popup: None,
            settings: Settings::new(),
        }
    }

    pub fn get_keybinding(&self) -> &KeyBindings {
        match self.running_state {
            RunningState::SelectSerialPort(_) => &self.settings.serialport_keybindings,
            RunningState::Main(_) => &self.settings.main_keybindings,
            RunningState::Configure(_) => todo!(),
            RunningState::GetInformation(_) => todo!(),
            RunningState::Quit(_) => todo!(),
            RunningState::ForceQuit => todo!(),
        }
    }
}

#[allow(dead_code)]
#[derive(strum::Display)]
pub enum RunningState {
    SelectSerialPort(SerialPortScreenState),
    Main(MainScreenState),
    Configure(ConfigureScreenState),
    GetInformation(GetInformationScreenState),
    Quit(QuitScreenState),
    ForceQuit,
}

#[derive(Debug, PartialEq)]
pub struct SerialPortScreenState {
    pub ports: Vec<SerialPortInfo>,
    pub last_selection: Option<usize>,
    pub list_state: ListState,
}

impl SerialPortScreenState {
    pub fn new() -> Self {
        Self {
            last_selection: None,
            ports: Vec::new(),
            list_state: ListState::default(),
        }
    }
}

impl Default for SerialPortScreenState {
    fn default() -> Self {
        Self::new()
    }
}

// #[derive(Debug)]
pub struct MainScreenState {
    pub port_name: String,
    pub reader: Box<dyn Read>,
    pub writer: Box<dyn Write>,
    pub list_state: ListState,
}

impl MainScreenState {
    pub fn with_serial_port(serial_port: Box<dyn SerialPort>) -> Self {
        Self {
            port_name: serial_port.name().unwrap(),
            reader: serial_port.try_clone().unwrap(),
            writer: serial_port,
            list_state: ListState::default(),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct ConfigureScreenState {}

#[derive(Debug, PartialEq, Default)]
pub struct GetInformationScreenState {}

#[derive(Debug, PartialEq, Default)]
pub struct QuitScreenState {}

#[derive(Debug)]
pub enum PopUpState {
    Message(String),
    ShowKeyBindings,
}

#[cfg(test)]
mod tests {
    use super::RunningState;

    #[test]
    fn test_enum_display() {
        let e = RunningState::SelectSerialPort(Default::default());
        let e1 = RunningState::Configure(Default::default());
        let e2 = RunningState::GetInformation(Default::default());
        let e3 = RunningState::ForceQuit;
        // let= RunningState::Main(Default::default());

        println!("{e}\n{e1}\n{e2}\n{e3}\n");
    }
}
