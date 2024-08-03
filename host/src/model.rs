use ratatui::widgets::ListState;
use serialport::SerialPortInfo;

#[derive(Debug)]
pub struct Model {
    pub running_state: RunningState,
}

impl Model {
    pub fn new() -> Self {
        Self {
            running_state: RunningState::SelectSerialPort(Default::default()),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    pub list_state: ListState
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

#[derive(Debug, PartialEq)]
pub struct MainScreenState {}

#[derive(Debug, PartialEq)]
pub struct ConfigureScreenState {}

#[derive(Debug, PartialEq)]
pub struct GetInformationScreenState {}

#[derive(Debug, PartialEq, Default)]
pub struct QuitScreenState {}
