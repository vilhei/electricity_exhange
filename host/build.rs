use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    str::FromStr,
};

use strum::{EnumMessage, VariantNames};

#[path = "src/action.rs"]
mod action;

const ACTION_INFO: &str =
    "# For each keybinding [action] must be set to one of the following values:\n";
const GENERATED_END: &str = "\n# Above is automatically generated comment by build process.\n\n";
const SETTINGS_FILE: &str = "./configs/settings.toml";

fn main() {
    // println!("cargo::rerun-if-changed=src/action.rs");
    let mut msg = String::from(ACTION_INFO);
    action::UiMessage::VARIANTS.iter().for_each(|v| {
        let mut s = String::from("# - ");
        s.push_str(v);

        let help_text = match action::UiMessage::from_str(v).unwrap().get_message() {
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
    });
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
    println!("file_content: {}", file_content);

    let end_idx = match file_content.find(GENERATED_END) {
        Some(idx) => idx + GENERATED_END.len(),
        None => 0,
    };
    file_content.replace_range(..end_idx, &msg);
    file.write_all(file_content.as_bytes()).unwrap();
}
