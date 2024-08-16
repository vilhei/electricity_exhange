use ratatui::{
    prelude::*,
    // style::Color,
    widgets::{Block, Borders},
};

// pub const LIST_BLOCK: ratatui::widgets::Block<'_> = Block::new().borders(Borders::ALL);

#[macro_export]
macro_rules! list_block {
    () => {
        Block::default().borders(Borders::ALL)
    };
}

// pub const TITLE_BLOCK: ratatui::widgets::Block<'_> = Block::new()
//     .borders(Borders::ALL)
//     // .bg(Color::Red)
//     .border_type(ratatui::widgets::BorderType::Rounded);

#[macro_export]
macro_rules! title_block {
    ($color:expr) => {
        Block::new()
            .borders(Borders::ALL)
            .bg($color)
            .border_type(ratatui::widgets::BorderType::Rounded)
    };
    () => {
        Block::new()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
    };
}
