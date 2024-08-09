use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    str::FromStr,
};

use action::Action;
use strum::{EnumMessage, VariantNames};

#[path = "src/action.rs"]
mod action;

impl Action {
    /// Returns true if Action can be binded to user desired keybind
    fn is_customizable(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match *self {
            Action::SelectLast => false,
            Action::SelectFirst => false,
            Action::ClosePopUp => false,
            Action::MustSelectOne => false,
            _ => true,
        }
    }
}

const ACTION_INFO: &str = "# Each keybinding must be set to one one the actions.
# And each action needs to be assigned to some keybinding
# Actions : \n";
const GENERATED_END: &str = "\n# Above is automatically generated comment by build process.\n\n";
const SETTINGS_FILE: &str = "./configs/settings.toml";

fn main() {
    println!("cargo::rerun-if-changed=src/action.rs");
    let mut msg = String::from(ACTION_INFO);

    for variant_name in action::Action::VARIANTS {
        let mut s = String::from("# - ");
        s.push_str(variant_name);
        let variant = action::Action::from_str(variant_name).unwrap();
        if !variant.is_customizable() {
            continue;
        }
        let help_text = match variant.get_message() {
            Some(m) => {
                let mut tmp = String::from(" : (");
                tmp.push_str(m);
                tmp.push(')');
                tmp
            }
            None => "".to_string(),
        };

        s.push_str(&help_text);
        s.push('\n');
        msg.push_str(&s);
    }

    msg.push_str(GENERATED_END);
    let mut file_content = String::new();
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(SETTINGS_FILE)
        .unwrap();
    File::read_to_string(&mut file, &mut file_content).unwrap();
    file.set_len(0).unwrap();
    file.rewind().unwrap();
    // println!("file_content: {}", file_content);

    let end_idx = match file_content.find(GENERATED_END) {
        Some(idx) => idx + GENERATED_END.len(),
        None => 0,
    };
    file_content.replace_range(..end_idx, &msg);
    file.write_all(file_content.as_bytes()).unwrap();
}
