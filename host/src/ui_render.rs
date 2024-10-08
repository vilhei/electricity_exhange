use crate::model::{
    ConfigureScreenState, GetInformationScreenState, MainScreenState, Model, PopUpState,
    QuitScreenState, RunningState, SerialPortScreenState,
};
use crossterm::event::KeyEvent;
use host::{
    action::Action, centered_rect, list_block, settings::keybindings::key_event_to_string,
    title_block,
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use serialport::SerialPortInfo;
use std::collections::HashMap;
use strum::VariantNames;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

pub fn view(model: &mut Model, f: &mut Frame) {
    let area = render_common_screen(f);

    match &mut model.running_state {
        RunningState::SelectSerialPort(state) => render_select_serialport_screen(state, f, &area),
        RunningState::Main(state) => render_main_screen(state, f, &area),
        RunningState::Configure(state) => render_configure_screen(state, f),
        RunningState::GetInformation(state) => render_get_information_screen(state, f),
        RunningState::Quit(state) => render_quit_screen(state, f),
        RunningState::ForceQuit => todo!(),
    }
    render_popup(model, f, Some(area));
}

fn render_common_screen(f: &mut Frame) -> Rect {
    let bg_color_block = Block::default().bg(Color::Indexed(237));
    f.render_widget(bg_color_block, f.size());

    let chunks =
        Layout::vertical([Constraint::Percentage(75), Constraint::Fill(1)]).split(f.size());

    let bottom_areas =
        Layout::horizontal([Constraint::Percentage(70), Constraint::Fill(1)]).split(chunks[1]);

    let log_widget = construct_tui_logger_widget();

    f.render_widget(log_widget, bottom_areas[0]);

    let block = Block::bordered()
        .title("Help")
        .title_alignment(Alignment::Center)
        .style(Style::default());

    let keybinding_help = Line::from(Span::raw(format!("{:<10} | show keybindings", "k")));
    let help_text = Text::from_iter([keybinding_help]);
    let help_text = Paragraph::new(help_text).block(block);

    f.render_widget(help_text, bottom_areas[1]);

    chunks[0]
}

fn render_popup(model: &Model, f: &mut Frame, target_area: Option<Rect>) {
    if let Some(p) = &model.popup {
        let mut area = target_area.unwrap_or(f.size());
        area = centered_rect(50, 50, area);
        f.render_widget(Clear, area);
        let block = Block::new()
            .bg(Color::Rgb(99, 168, 159))
            // .fg(Color::Red)rgb(224, 226, 110)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .title("Popup");
        f.render_widget(block, area);

        match p {
            PopUpState::Message(m) => render_popup_message(area, f, m),
            PopUpState::ShowKeyBindings => render_keybindings(area, f, model.get_keybinding()),
        }
    }
}

fn render_popup_message(area: Rect, f: &mut Frame, m: &str) {
    let msg_lines = m
        .lines()
        .map(|line| Line::from(Span::styled(line, Style::default()).fg(Color::White)));

    let mut msg_text = Text::from_iter(msg_lines);
    let exit_text = Line::from(Span::styled(
        "Press [Esc] to close the popup",
        Style::default(),
    ));

    msg_text.push_line(exit_text);

    let msg = Paragraph::new(msg_text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(msg, area);
}

fn render_keybindings<K>(area: Rect, f: &mut Frame, m: K)
where
    K: AsRef<HashMap<KeyEvent, Action>>,
{
    let list_items: Vec<ListItem> = m
        .as_ref()
        .iter()
        .map(|(k, a)| {
            ListItem::new(Line::from(vec![Span::styled(
                format!("{:<15} | {}", key_event_to_string(k), a.as_ref()),
                Style::default(),
            )]))
        })
        .collect();

    let list = List::new(list_items);
    f.render_widget(list, area);
}

fn render_select_serialport_screen(state: &mut SerialPortScreenState, f: &mut Frame, area: &Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(*area);

    let title = Paragraph::new(Span::styled(
        "Select port connected to esp32c3",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(title_block!());

    f.render_widget(title, chunks[0]);

    let list_items: Vec<ListItem> = state.ports.iter().map(serial_port_to_list_item).collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .block(list_block!());
    // f.render_widget(list, chunks[1]);
    f.render_stateful_widget(list, chunks[1], &mut state.list_state);
}

fn serial_port_to_list_item<'a>(p: &SerialPortInfo) -> ListItem<'a> {
    match &p.port_type {
        serialport::SerialPortType::UsbPort(port_info) => ListItem::new(Line::from(Span::styled(
            format!(
                "{: <15} | {:<15} | {:<15}",
                p.port_name,
                port_info.manufacturer.clone().unwrap_or("".to_string()),
                port_info.product.clone().unwrap_or("".to_string())
            ),
            Style::default(),
        ))),
        _ => ListItem::new(Line::from(Span::styled(
            p.port_name.clone(),
            Style::default(),
        ))),
    }
}

fn render_main_screen(state: &mut MainScreenState, f: &mut Frame, area: &Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(*area);

    let title = Paragraph::new(Span::styled(
        format!("Main View - selected port : {}", state.port_name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(title_block!());
    f.render_widget(title, chunks[0]);

    let list_items: Vec<ListItem> = shared::Message::VARIANTS
        .iter()
        .map(|v| ListItem::new(Line::from(Span::styled(*v, Style::default()))))
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .block(list_block!());
    f.render_stateful_widget(list, chunks[1], &mut state.list_state);
}

#[allow(unused)]
fn render_configure_screen(state: &ConfigureScreenState, f: &mut Frame) {
    todo!()
}

#[allow(unused)]
fn render_get_information_screen(state: &GetInformationScreenState, f: &mut Frame) {
    todo!()
}

#[allow(unused)]
fn render_quit_screen(state: &QuitScreenState, f: &mut Frame) {
    todo!()
}

fn construct_tui_logger_widget<'a>() -> tui_logger::TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .block(
            Block::bordered().title(Title::from(
                Line::styled("Logs", Style::default()).alignment(Alignment::Center),
            )), // .bg(Color::Green),
        )
        .output_separator('|')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(true)
        .output_file(false)
        .output_line(false)
        .style_trace(Style::default().fg(Color::White))
        .style_info(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::LightYellow))
        .style_error(Style::default().fg(Color::LightRed))
}
