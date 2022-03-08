use num_format::{Locale, ToFormattedString};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::app::{App, ChannelStats};

pub fn draw_hosted<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(area);

    let toprow = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Percentage(80)].as_ref())
        .split(vchunks[0]);

    draw_info(f, app, toprow[0]);
    draw_active_chans(f, app, toprow[1]);
    draw_relays_amounts(f, app, vchunks[1]);
    draw_relays_volumes(f, app, vchunks[2]);
}

fn draw_info<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let tittles = vec![
        Spans::from("Channels activity"),
        Spans::from("Channels volume"),
        Spans::from(vec![Span::from("Active:")]),
        Spans::from(vec![Span::from("Suspended:")]),
        Spans::from(vec![Span::from("Offline:")]),
        Spans::from(""),
        Spans::from("Relayed"),
        Spans::from(vec![Span::from("per day:")]),
        Spans::from(vec![Span::from("per month:")]),
        Spans::from(vec![Span::from("per day:")]),
        Spans::from(vec![Span::from("per month:")]),
        Spans::from(vec![Span::from("percent:")]),
        Spans::from(""),
        Spans::from("Fees"),
        Spans::from(vec![Span::from("per day:")]),
        Spans::from(vec![Span::from("per month:")]),
        Spans::from(vec![Span::from("ARP year:")]),
    ];
    let block = Block::default()
        .title("Stats")
        .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT);
    let titles_paragraph = Paragraph::new(tittles)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(titles_paragraph, hchunks[0]);

    let values = vec![
        Spans::from(vec![
            Span::styled(
                format!("{:?}", app.active_chans),
                Style::default().fg(Color::Green),
            ),
            Span::from("/"),
            Span::styled(
                format!("{:?}", app.pending_chans),
                Style::default().fg(Color::Yellow),
            ),
            Span::from("/"),
            Span::styled(
                format!("{:?}", app.sleeping_chans),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.active_sats / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.pending_sats / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Yellow),
        )]),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.sleeping_sats / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Gray),
        )]),
        Spans::from(""),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            app.relayed_count_day.to_formatted_string(&Locale::en),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            app.relayed_count_month.to_formatted_string(&Locale::en),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.relayed_day / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.relayed_month / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!("{:.2}%", app.relayed_percent()),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(""),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.fee_day / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!(
                "{} sats",
                (app.fee_month / 1000).to_formatted_string(&Locale::en)
            ),
            Style::default().fg(Color::Green),
        )]),
        Spans::from(vec![Span::styled(
            format!("{:.2}%", app.return_rate),
            Style::default().fg(Color::Green),
        )]),
    ];
    let block = Block::default().borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT);
    let values_paragraph = Paragraph::new(values)
        .block(block)
        .alignment(Alignment::Right);
    f.render_widget(values_paragraph, hchunks[1]);
}

fn draw_active_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let headbody = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)].as_ref())
        .split(area);

    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(headbody[1]);

    let vchunks: Vec<Vec<Rect>> = hchunks
        .iter()
        .map(|column| {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                )
                .split(*column)
        })
        .collect();

    let chans_in_column = 4;
    let chans_to_draw = chans_in_column * vchunks.len();
    let chans_to_skip = app.channels_page as usize * chans_to_draw;
    let mut chans = app.hosted_stats.clone();
    chans.sort_by(|a, b| b.relays_volume.partial_cmp(&a.relays_volume).unwrap());
    for (i, c) in chans
        .iter()
        .skip(chans_to_skip)
        .take(chans_to_draw)
        .enumerate()
    {
        draw_active_chan(f, vchunks[i / chans_in_column][i % chans_in_column], c);
    }
}

fn draw_active_chan<B: Backend>(f: &mut Frame<B>, area: Rect, chan: &ChannelStats) {
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

    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vchunks[2]);

    let chan_tittle = vec![Spans::from(vec![Span::styled(
        chan.alias.clone(),
        Style::default().fg(Color::White),
    )])];
    let paragraph = Paragraph::new(chan_tittle).alignment(Alignment::Left);
    f.render_widget(paragraph, vchunks[0]);

    let capacity = chan.local + chan.remote;

    let channel_ratio = if capacity == 0 {
        0.0
    } else {
        chan.local as f64 / capacity as f64
    };
    let local = (chan.local / 1000).to_formatted_string(&Locale::en);
    let remote = (chan.remote / 1000).to_formatted_string(&Locale::en);
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Blue)
                .bg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )
        .ratio(channel_ratio)
        .label(format!("{}/{}", local, remote));
    f.render_widget(gauge, vchunks[1]);

    let col0_spans = vec![
        Spans::from(vec![
            Span::from("Relays: ".to_owned()),
            Span::styled(
                format!("{}", chan.relays_amount),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::from("Fees: ".to_owned()),
            Span::styled(
                (chan.relays_fees / 1000).to_formatted_string(&Locale::en),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];
    let stats_col0 = Paragraph::new(col0_spans).alignment(Alignment::Left);
    f.render_widget(stats_col0, hchunks[0]);

    let col1_spans = vec![Spans::from(vec![
        Span::from("Volume: ".to_owned()),
        Span::styled(
            (chan.relays_volume / 1000).to_formatted_string(&Locale::en),
            Style::default().fg(Color::Gray),
        ),
    ])];
    let stats_col1 = Paragraph::new(col1_spans).alignment(Alignment::Left);
    f.render_widget(stats_col1, hchunks[1]);
}

fn draw_relays_amounts<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    "24h relay count (max: {})",
                    app.relays_maximum_count
                ))
                .borders(Borders::LEFT | Borders::RIGHT),
        )
        .data(&app.relays_amounts_line)
        .style(Style::default().fg(Color::Red));
    f.render_widget(sparkline, area);
}

fn draw_relays_volumes<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    "24h relay volumes, (max: {} sats)",
                    (app.relays_maximum_volume / 1000).to_formatted_string(&Locale::en)
                ))
                .borders(Borders::LEFT | Borders::RIGHT),
        )
        .data(&app.relays_volumes_line)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(sparkline, area);
}
