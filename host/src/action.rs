use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[derive(
    Debug,
    Serialize,
    strum::EnumString,
    strum::VariantNames,
    strum::EnumMessage,
    strum::AsRefStr,
    Copy,
    Clone,
)]

pub enum Action {
    #[strum(message = "Lists available serial ports")]
    FetchSerialPorts,
    #[strum(message = "Quits the application without saving anything or checking with user")]
    ForceQuit,
    SelectionUp,
    SelectionDown,
    ClearSelection,
    SelectLast,
    SelectFirst,
    #[strum(message = "Connect to selected serial port and continue")]
    StateChangeFromSerialPortToMain,
    MustSelectOne,
    ClosePopUp,
    #[strum(message = "Show keybindings, these can be configured in the settings.toml file")]
    ShowKeyBindings,
    SerialPortConnectionFail,
}

/// Implemented only to get error message with list of acceptable enum variants
/// Probably not most efficient solution but does not really matter when
/// configuration file is realistically deserialized once per program run
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer).unwrap();
        Action::from_str(&s)
            .map_err(|e| serde::de::Error::unknown_variant(&e.to_string(), Action::VARIANTS))
    }
}

#[cfg(test)]
mod tests {}
