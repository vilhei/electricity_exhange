use host::settings::{keybindings::KeyBindings, Settings};
use ratatui::widgets::ListState;
use serialport::SerialPortInfo;

#[derive(Debug)]
pub struct Model<'a> {
    pub running_state: RunningState,
    pub popup: Option<PopUpState<'a>>,
    pub settings: Settings,
}

impl Model<'_> {
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
            RunningState::Main(_) => todo!(),
            RunningState::Configure(_) => todo!(),
            RunningState::GetInformation(_) => todo!(),
            RunningState::Quit(_) => todo!(),
            RunningState::ForceQuit => todo!(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, strum::Display)]
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

#[derive(Debug, PartialEq, Default)]
pub struct MainScreenState {
    pub port_name: String,
}

#[derive(Debug, PartialEq, Default)]
pub struct ConfigureScreenState {}

#[derive(Debug, PartialEq, Default)]
pub struct GetInformationScreenState {}

#[derive(Debug, PartialEq, Default)]
pub struct QuitScreenState {}

#[derive(Debug)]
pub enum PopUpState<'a> {
    Message(&'a str),
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
        let e4 = RunningState::Main(Default::default());

        println!("{e}\n{e1}\n{e2}\n{e3}\n{e4}");
    }
}
