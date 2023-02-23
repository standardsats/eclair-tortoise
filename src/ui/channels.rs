use num_format::{Locale, ToFormattedString};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use std::iter;

use crate::app::{App, ChannelStats};
use crate::api::{
    channel::{ChannelInfo, ChannelState},
};

pub fn draw_channels<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {

    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(66),
            ]
            .as_ref(),
        )
        .split(area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(hchunks[0]);

    let titles = vec![
        Spans::from(vec![
            Span::styled("A", Style::default().fg(Color::Yellow)),
            Span::styled("ctive", Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::styled("P", Style::default().fg(Color::Green)),
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::styled("nding", Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::styled("S", Style::default().fg(Color::Yellow)),
            Span::styled("leeping", Style::default().fg(Color::Green)),
        ])
    ];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.chans_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, vchunks[0]);
    match app.chans_tab {
        0 => draw_active_chans(f, app, vchunks[1]),
        1 => draw_closing_chans(f, app, vchunks[1]),
        2 => draw_sleeping_chans(f, app, vchunks[1]),
        _ => (),
    }
}

const CHANNEL_ITEM_SIZE: usize = 1;

fn draw_active_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let visible_count = (area.height as usize - 2)/CHANNEL_ITEM_SIZE;
    let vchunks_sizes: Vec<Constraint> = iter::repeat(Constraint::Length(CHANNEL_ITEM_SIZE as u16)).take(visible_count as usize).collect();
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vchunks_sizes)
        .split(area);

    let block = Block::default().title("Active").borders(Borders::ALL);
    f.render_widget(block, area);

    let mut chans: Vec<&ChannelStats> = app.channels_stats.iter().filter(|c| app.channels[c.info_id].state == ChannelState::Normal).collect();
    chans.sort_by(|a, b| b.volume().partial_cmp(&a.volume()).unwrap());
    for (i, c) in chans.iter().take(visible_count as usize).enumerate() {
        draw_channel(f, vchunks[i], c);
    }
}

fn draw_closing_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let visible_count = (area.height as usize - 2)/CHANNEL_ITEM_SIZE;
    let vchunks_sizes: Vec<Constraint> = iter::repeat(Constraint::Length(CHANNEL_ITEM_SIZE as u16)).take(visible_count as usize).collect();
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vchunks_sizes)
        .split(area);

    let block = Block::default().title("Pending").borders(Borders::ALL);
    f.render_widget(block, area);

    let mut chans: Vec<&ChannelStats> = app.channels_stats.iter().filter(|c| app.channels[c.info_id].state.is_pending()).collect();
    chans.sort_by(|a, b| b.volume().partial_cmp(&a.volume()).unwrap());
    for (i, c) in chans.iter().take(visible_count as usize).enumerate() {
        draw_channel(f, vchunks[i], c);
    }
}

fn draw_sleeping_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let visible_count = (area.height as usize - 2)/CHANNEL_ITEM_SIZE;
    let vchunks_sizes: Vec<Constraint> = iter::repeat(Constraint::Length(CHANNEL_ITEM_SIZE as u16)).take(visible_count as usize).collect();
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vchunks_sizes)
        .split(area);

    let block = Block::default().title("Sleeping").borders(Borders::ALL);
    f.render_widget(block, area);

    let mut chans: Vec<&ChannelStats> = app.channels_stats.iter().filter(|c| app.channels[c.info_id].state.is_sleeping()).collect();
    chans.sort_by(|a, b| b.volume().partial_cmp(&a.volume()).unwrap());
    for (i, c) in chans.iter().take(visible_count as usize).enumerate() {
        draw_channel(f, vchunks[i], c);
    }
}

fn draw_channel<B: Backend>(f: &mut Frame<B>, area: Rect, chan: &ChannelStats) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(CHANNEL_ITEM_SIZE as u16),
            ]
            .as_ref(),
        )
        .split(area);


    let chan_tittle = vec![Spans::from(vec![Span::styled(
        chan.alias.clone(),
        Style::default().fg(Color::White),
    )])];
    let paragraph = Paragraph::new(chan_tittle).alignment(Alignment::Left);
    f.render_widget(paragraph, vchunks[0]);
}