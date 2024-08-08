pub mod keybindings;

use keybindings::KeyBindings;
use serde::Deserialize;
use tracing::{info, instrument, Level};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub serialport_keybindings: KeyBindings,
}

impl Settings {
    #[instrument(ret(level=Level::TRACE))]
    pub fn new() -> Self {
        info!("Creating new settings object");
        let config_path = "./configs/settings.toml";
        let file = config::File::with_name(config_path);
        let settings = config::Config::builder().add_source(file).build().unwrap();
        settings.try_deserialize().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use config::Config;

    #[test]
    fn deserialize_keybindings() {
        let config_path = "./configs/settings.toml";
        let file = config::File::with_name(config_path);
        let settings = Config::builder().add_source(file).build().unwrap();
        let settings: Settings = settings.try_deserialize().unwrap();

        for (key, action) in settings.serialport_keybindings.0.into_iter() {
            println!("{:?} -  {:?}", key.code, action,);
        }
    }
}
