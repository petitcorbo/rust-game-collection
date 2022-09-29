use std::{io, format, time::{Duration, Instant}};
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    layout::{Layout, Constraint},
    style::{Color, Style},
    text::Span,
    symbols,
    Terminal
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::size,
};

const HELP: &str = "[s]: 'swap cell state', [p]: 'pause/resume game', [c]: 'clear grid', [arrows]: 'move cursor'";


fn get_next_gen(grid: &Vec<Vec<i8>>) -> Vec<Vec<i8>> {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut next_gen: Vec<Vec<i8>> = vec![vec![0; cols]; rows];

    for y in 0..rows {
        for x in 0..cols {
            let cell_state = grid[y][x];

            // count neighbors \\
            let mut live_neighbors = -cell_state;
            for i in -1i16..=1 {
                for j in -1i16..=1 {
                    let new_x = (x as i16) + i;
                    let new_y = (y as i16) + j;

                    if new_x > 0 && new_y > 0 && new_x < cols as i16 && new_y < rows as i16 {
                        live_neighbors += grid[new_y as usize][new_x as usize];
                    }
                }
            }

            // underpopulation \\
            if cell_state == 1 && live_neighbors < 2 {
                next_gen[y][x] = 0;
            }
            // overpopulation \\
            else if cell_state == 1 && live_neighbors > 3 {
                next_gen[y][x] = 0;
            }
            // reproduction \\
            else if cell_state == 0 && live_neighbors == 3 {
                next_gen[y][x] = 1;
            }
            // stable population \\
            else {
                next_gen[y][x] = cell_state;
            }
        }
    }
    // return \\
    next_gen
}


pub fn run_gol<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // grid creation \\
    let (c, r) = size().unwrap();
    let (cols, rows) = ((c-2) as usize, (r-5) as usize);
    let mut new_gen: Vec<Vec<i8>> = vec![vec![0; cols]; rows];
    let mut mid_gen: Vec<(f64, f64)> = Vec::new();
    let mut old_gen: Vec<(f64, f64)> = Vec::new();

    let mut paused: bool = true;
    let mut history_on: bool = false;
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);
    let mut frame_rate = Duration::from_millis(400);
    let mut cursor_x = cols / 2;
    let mut cursor_y = rows / 2;
    let mut running_time = 0.0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(2)])
                .split(f.size());
            
            // controls information \\
            let paragraph = Paragraph::new(HELP)
                .block(Block::default().title("[Help]").borders(Borders::ALL));
            f.render_widget(paragraph, chunks[0]);
            
            // canvas \\
            let pause_color = if paused {Color::Green} else {Color::Green};
            let title = vec![
                Span::raw("[Game of Life: "),
                Span::styled("playing", Style::default().fg(pause_color)),
                Span::raw(format!("| Timer: {}]", running_time as u16))
            ];
            let canvas = Canvas::default()
                .block(Block::default().title(title).borders(Borders::ALL))
                .x_bounds([0.0, (cols-1) as f64])
                .y_bounds([0.0, (rows-1) as f64])
                .marker(symbols::Marker::Block)
                .paint(|ctx| {
                    if history_on {
                        for (x, y) in old_gen.clone() {
                            ctx.print(x as f64, y as f64, Span::styled("█", Style::default().fg(Color::Rgb(0, 50, 50))))
                        }
                        for (x, y) in mid_gen.clone() {
                            ctx.print(x as f64, y as f64, Span::styled("█", Style::default().fg(Color::Rgb(0, 100, 100))))
                        }
                    }
                    for y in 0..rows {
                        for x in 0..cols {
                            if new_gen[y][x] == 1 {
                                ctx.print(x as f64, y as f64, Span::styled("█", Style::default().fg(Color::Rgb(0, 255, 255))))
                            }
                        }
                    }
                    ctx.print(cursor_x as f64, cursor_y as f64, Span::styled("█", Style::default().fg(Color::White)))
                });
            f.render_widget(canvas, chunks[1]);
        })?;

        // time update \\
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // input handler \\
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('h') => history_on = !history_on,
                    KeyCode::Char('c') => {
                        new_gen = vec![vec![0; cols]; rows];
                        paused = true;
                        running_time = 0.0;
                    }
                    KeyCode::Char('s') => {
                        new_gen[cursor_y][cursor_x] = 1 - new_gen[cursor_y][cursor_x]
                    }
                    KeyCode::Enter => {
                        new_gen[cursor_y][cursor_x] = 1 - new_gen[cursor_y][cursor_x]
                    }
                    KeyCode::Left => if cursor_x > 0 {cursor_x -= 1},
                    KeyCode::Right => if cursor_x < (cols-1) {cursor_x += 1},
                    KeyCode::Up => if cursor_y < (rows-1) {cursor_y += 1},
                    KeyCode::Down => if cursor_y > 0 {cursor_y -= 1},
                    KeyCode::Char('0') => frame_rate = Duration::from_millis(50),
                    KeyCode::Char('1') => frame_rate = Duration::from_millis(100),
                    KeyCode::Char('2') => frame_rate = Duration::from_millis(200),
                    KeyCode::Char('3') => frame_rate = Duration::from_millis(300),
                    KeyCode::Char('4') => frame_rate = Duration::from_millis(400),
                    KeyCode::Char('5') => frame_rate = Duration::from_millis(1000),
                    _ => {}
                }
            }
        }

        // grid update \\
        if last_tick.elapsed() >= frame_rate {
            if !paused {
                old_gen = mid_gen.clone();
                mid_gen = Vec::new();
                for y in 0..rows {
                    for x in 0..cols {
                        if new_gen[y][x] == 1 {
                            mid_gen.push((x as f64, y as f64));
                        }
                    }
                }
                new_gen = get_next_gen(&new_gen);
                running_time += 0.4
            }
            last_tick = Instant::now();
        }
    }
}
