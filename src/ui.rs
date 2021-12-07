use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, thread, time::Duration, sync::mpsc};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame, Terminal,
};

use super::app::{AppMutex, App};

pub fn run_ui(app: AppMutex) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(res?)
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
    thread::spawn(move || {
        if let Ok(Event::Key(key)) = event::read() {
            if let Err(err) = keys_tx.send(AppEvent::Input(key)) {
                eprintln!("{}", err);
                return;
            }
        }
    });
    thread::spawn(move || loop {
        if let Err(err) = tx.send(AppEvent::Tick) {
            eprintln!("{}", err);
            break;
        }
        thread::sleep(tick_rate);
    });
    rx
}

fn ui<B: Backend>(f: &mut Frame<B>, mapp: AppMutex) {
    let size = f.size();
    let app = mapp.lock().unwrap();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
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
        let block = Block::default().title("Errors occured").borders(Borders::ALL);
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
        .margin(1)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(area);

    let toprow = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
        .split(vchunks[0]);

    draw_info(f, app, toprow[0]);
}

fn draw_info<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let node_info = vec![
        Spans::from(vec![
            Span::from(" Node:           "),
            Span::styled(
                app.node_info.alias.clone(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::from(" Network:        "),
            Span::from(format!("{:?}", app.node_info.network)),
        ]),
        Spans::from(vec![
            Span::from(" Active chans:   "),
            Span::styled(
                format!("{:?}", app.active_chans),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::from(" Pending chans:  "),
            Span::styled(
                format!("{:?}", app.pending_chans),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Spans::from(vec![
            Span::from(" Sleeping chans: "),
            Span::styled(
                format!("{:?}", app.sleeping_chans),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];
    let block = Block::default().title("Stats").borders(Borders::ALL);
    let paragraph = Paragraph::new(node_info)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
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
