#![feature(panic_payload_as_str)]
use std::io::stdout;

use log::info;
use ratatui::{
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
};

pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

pub fn restore_terminal() -> color_eyre::Result<()> {
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

// TODO figure out how to log error when panicking
pub fn install_panic_hook() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .display_location_section(true)
        .display_env_section(true)
        .into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    eyre_hook.install()?;

    info!("Installing custom panic hook");

    std::panic::set_hook(Box::new(move |info| {
        // let bt = Backtrace::capture();
        tracing::error!("{}", info);
        let _ = restore_terminal();
        panic_hook(info);
    }));
    Ok(())
}

// pub fn install_panic_hook() -> color_eyre::Result<()> {
//     let (panic, error) = HookBuilder::default().into_hooks();
//     let panic = panic.into_panic_hook();
//     let error = error.into_eyre_hook();
//     info!("Installing custom panic hook");

//     color_eyre::eyre::set_hook(Box::new(move |e| {
//         tracing::trace!("{}", "trace()");
//         let _ = restore_terminal();
//         error(e)
//     }))?;
//     std::panic::set_hook(Box::new(move |info| {
//         let _ = restore_terminal();
//         panic(info);
//     }));
//     Ok(())
// }

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
