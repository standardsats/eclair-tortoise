use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use num_format::{Locale, ToFormattedString};
use std::{error::Error, io, sync::mpsc, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Sparkline, Tabs, Gauge},
    Frame, Terminal,
};
use log::*;

use super::app::{App, AppMutex, ChannelStats};

pub fn run_ui(app: AppMutex) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    // restore terminal
    defer! {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend).unwrap();
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        terminal.show_cursor().unwrap();
    }

    // Run the app
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    run_app(&mut terminal, app)?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mapp: AppMutex) -> io::Result<()> {
    let events = events(Duration::from_secs_f32(1.0));
    loop {
        terminal.draw(|f| ui(f, mapp.clone()))?;

        match events.recv().unwrap() {
            AppEvent::Input(key) => {
                let mut app = mapp.lock().unwrap();
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Right => app.next_tab(),
                    KeyCode::Left => app.previous_tab(),
                    KeyCode::Enter if !app.errors.is_empty() => app.errors = vec![],
                    _ => app.react_hotkey(key.code),
                }
            }
            AppEvent::Tick => (),
        }
    }
}

enum AppEvent {
    Input(KeyEvent),
    Tick,
}

fn events(tick_rate: Duration) -> mpsc::Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();
    let keys_tx = tx.clone();
    thread::spawn(move || loop {
        if let Ok(Event::Key(key)) = event::read() {
            if let Err(err) = keys_tx.send(AppEvent::Input(key)) {
                error!("{}", err);
                return;
            }
        }
    });
    thread::spawn(move || loop {
        if let Err(err) = tx.send(AppEvent::Tick) {
            error!("{}", err);
            break;
        }
        thread::sleep(tick_rate);
    });
    rx
}

fn ui<B: Backend>(f: &mut Frame<B>, mapp: AppMutex) {
    let size = f.size();
    let mut app = mapp.lock().unwrap();
    app.resize(size.width);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);
    let titles = app
        .tabs
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.tab_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);
    match app.tab_index {
        0 => draw_dashboard(f, &app, chunks[1]),
        1 => draw_peers(f, &app, chunks[1]),
        2 => draw_onchain(f, &app, chunks[1]),
        3 => draw_routing(f, &app, chunks[1]),
        _ => unreachable!(),
    };

    if !app.errors.is_empty() {
        let errors: Vec<Spans> = app.errors.iter().map(|e| Spans::from(e.clone())).collect();
        let block = Block::default()
            .title("Errors occured")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(errors)
            .block(block)
            .alignment(Alignment::Left);
        let area = centered_rect(80, 50, size);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(paragraph, area);
    }
}

fn draw_dashboard<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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
        Spans::from(vec![
            Span::from("Node:"),
        ]),
        Spans::from(vec![
            Span::from("Network:"),
        ]),
        Spans::from(""),

        // Spans::from("Channels"),
        // Spans::from(vec![
        //     Span::from("Active:"),
        // ]),
        // Spans::from(vec![
        //     Span::from("Pending:"),
        // ]),
        // Spans::from(vec![
        //     Span::from("Sleeping:"),
        // ]),
        // Spans::from(""),

        Spans::from("Channels volumes"),
        Spans::from(vec![
            Span::from("Active:"),
        ]),
        Spans::from(vec![
            Span::from("Pending:"),
        ]),
        Spans::from(vec![
            Span::from("Sleeping:"),
        ]),
        Spans::from(""),

        Spans::from("Relayed"),
        Spans::from(vec![
            Span::from("per day:"),
        ]),
        Spans::from(vec![
            Span::from("per mounth:"),
        ]),
        Spans::from(vec![
            Span::from("per day:"),
        ]),
        Spans::from(vec![
            Span::from("per mounth:"),
        ]),
        Spans::from(vec![
            Span::from("percent:"),
        ]),
        Spans::from(""),

        Spans::from("Fees"),
        Spans::from(vec![
            Span::from("per day:"),
        ]),
        Spans::from(vec![
            Span::from("per mounth:"),
        ]),
        Spans::from(vec![
            Span::from("ARP year:"),
        ]),
    ];
    let block = Block::default().title("Stats").borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT);
    let titles_paragraph = Paragraph::new(tittles)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(titles_paragraph, hchunks[0]);

    let values = vec![
        Spans::from(vec![
            Span::styled(
                app.node_info.alias.clone(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::from(format!("{:?}", app.node_info.network)),
        ]),
        Spans::from(""),

        // Spans::from(""),
        // Spans::from(vec![
        //     Span::styled(
        //         format!("{:?}", app.active_chans),
        //         Style::default().fg(Color::Green),
        //     ),
        // ]),
        // Spans::from(vec![
        //     Span::styled(
        //         format!("{:?}", app.pending_chans),
        //         Style::default().fg(Color::Yellow),
        //     ),
        // ]),
        // Spans::from(vec![
        //     Span::styled(
        //         format!("{:?}", app.sleeping_chans),
        //         Style::default().fg(Color::Gray),
        //     ),
        // ]),
        // Spans::from(""),

        Spans::from(""),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.active_sats / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.pending_sats / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.sleeping_sats / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(""),

        Spans::from(""),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{}",
                    app.relayed_count_day.to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{}",
                    app.relayed_count_mounth.to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.relayed_day / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.relayed_mounth / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("{:.2}%", app.relayed_percent()),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(""),

        Spans::from(""),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.fee_day / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!(
                    "{} sats",
                    (app.fee_mounth / 1000).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("{:.2}%", app.return_rate),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];
    let block = Block::default().borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT);
    let values_paragraph = Paragraph::new(values)
        .block(block)
        .alignment(Alignment::Right);
    f.render_widget(values_paragraph, hchunks[1]);
}

fn draw_active_chans<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(area);

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
    let mut chans = app.channels_stats.clone();
    chans.sort_by(|a, b| b.relays_volume.partial_cmp(&a.relays_volume).unwrap());
    for (i, c) in chans.iter().take(chans_in_column * vchunks.len()).enumerate() {
        draw_active_chan(f, app, vchunks[i / chans_in_column][i % chans_in_column], c);
    }
}

fn draw_active_chan<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect, chan: &ChannelStats) {
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

    let channel_ratio = chan.local as f64 / (chan.local + chan.remote) as f64;
    let local = (chan.local/1000).to_formatted_string(&Locale::en);
    let remote = (chan.remote/1000).to_formatted_string(&Locale::en);
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
            Span::styled(format!("{}", chan.relays_amount), Style::default().fg(Color::Gray)),
        ]),
        Spans::from(vec![
            Span::from("Fees: ".to_owned()),
            Span::styled((chan.relays_fees / 1000).to_formatted_string(&Locale::en), Style::default().fg(Color::Green)),
        ])
    ];
    let stats_col0 = Paragraph::new(col0_spans).alignment(Alignment::Left);
    f.render_widget(stats_col0, hchunks[0]);

    let col1_spans = vec![
        Spans::from(vec![
            Span::from("Volume: ".to_owned()),
            Span::styled((chan.relays_volume / 1000).to_formatted_string(&Locale::en), Style::default().fg(Color::Gray)),
        ])
    ];
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

fn draw_peers<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Peers").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_onchain<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Onchain").borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_routing<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Routing").borders(Borders::ALL);
    f.render_widget(block, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
