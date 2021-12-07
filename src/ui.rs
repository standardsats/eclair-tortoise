use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use super::app::App;

pub fn run_ui(app: App) -> Result<(), Box<dyn Error>> {
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Right => app.next_tab(),
                KeyCode::Left => app.previous_tab(),
                _ => app.react_hotkey(key.code),
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
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
        0 => draw_dashboard(f, app, chunks[1]),
        1 => draw_peers(f, app, chunks[1]),
        2 => draw_onchain(f, app, chunks[1]),
        3 => draw_routing(f, app, chunks[1]),
        _ => unreachable!(),
    };
}

fn draw_dashboard<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().title("Dashboard").borders(Borders::ALL);

    let node_info = vec![
        Spans::from(vec![
            Span::from(" Node:    "),
            Span::styled(
                app.node_info.alias.clone(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::from(" Network: "),
            Span::from(format!("{:?}", app.node_info.network)),
        ]),
    ];
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
