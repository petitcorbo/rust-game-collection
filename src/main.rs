mod game_of_life;
use crate::game_of_life::run_gol;
mod snake;
use crate::snake::run_snake;
mod cube;
use crate::cube::run_cube;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
    text::{Span, Spans},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

const GAMES: [&str; 4] = [
        "Game of Life",
        "Snake",
        "Cube",
        "Sudoku"
];

const DESCRIPTION: [&str; 4] = [
    "Conway's Game of Life:
-Underpopulation: Any live cell with fewer than two live neighbours dies.
-Stable population: Any live cell with two or three live neighbours lives on to the next generation.
-Overpopulation: Any live cell with more than three live neighbours dies.
-Reproduction: Any dead cell with exactly three live neighbours becomes a live cell.",
    "Snake:
    Control a snake, eat apples but not yourself and don't crash into walls !",
    "Cube:
    Rotate a 3D rendered cube.",
    ""
];


fn start_game<B: Backend>(terminal: &mut Terminal<B>, idx: &mut usize) -> io::Result<()> {
    match idx {
        0 => run_gol(terminal),
        1 => run_snake(terminal),
        2 => run_cube(terminal),
        _ => Ok(()),
    }
}


fn run<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut list_idx = 0;
    let mut list_state = ListState::default();

    loop {
        terminal.draw(|f| { // f as frame \\
            let chunks = Layout::default()
                .constraints([Constraint::Length(25), Constraint::Min(2)])
                .direction(Direction::Horizontal)
                .split(f.size());
    
            let list_items: Vec<ListItem> = GAMES
                .to_vec()
                .iter()
                .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
                .collect();
            list_state.select(Some(list_idx));
            let list = List::new(list_items)
                .block(Block::default().title("[Games]").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Cyan))
                .highlight_symbol(">");
            f.render_stateful_widget(list, chunks[0], &mut list_state);
            
            let paragraph = Paragraph::new(DESCRIPTION[list_idx])
                .block(Block::default().title("[Description]").borders(Borders::ALL))
                .wrap(Wrap {trim: true});
            f.render_widget(paragraph, chunks[1]);
        }).ok(); // TODO error handling

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => start_game(terminal, &mut list_idx),
                KeyCode::Up => {if list_idx > 0 {list_idx -= 1}; Ok(())},
                KeyCode::Down => {if list_idx < GAMES.len() - 1 {list_idx += 1}; Ok(())},
                _ => {Ok(())}
            }.ok(); // TODO error handling
        }
    }
}

fn main() -> Result<(), io::Error> {
    // setup terminal \\
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // start ui \\
    let res = run(&mut terminal);

    // restore terminal \\
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {println!("{:?}", err)}

    Ok(())
}
