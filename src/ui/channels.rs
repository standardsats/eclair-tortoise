use num_format::{Locale, ToFormattedString};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Sparkline, Gauge},
    Frame,
};
use std::iter;

use crate::app::{App, ChannelStats};
use crate::client::{
    channel::{ChannelInfo, ChannelState},
};

pub fn draw_channels<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(area);

    draw_active_chans(f, app, hchunks[0]);
    draw_closing_chans(f, app, hchunks[1]);
    draw_sleeping_chans(f, app, hchunks[2]);
}

fn draw_active_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let visible_count = (area.height-2) / 3;
    let vchunks_sizes: Vec<Constraint> = iter::repeat(Constraint::Length(3)).take(visible_count as usize).collect();
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
    let block = Block::default().title("Closing").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_sleeping_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Sleeping").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_channel<B: Backend>(f: &mut Frame<B>, area: Rect, chan: &ChannelStats) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
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