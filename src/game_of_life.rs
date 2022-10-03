use std::{io, format, time::{Duration, Instant}};
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    layout::{Layout, Constraint},
    style::{Color, Style},
    text::Span,
    symbols,
    Frame,
    Terminal
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::size,
};

const HELP: &str = "[s]: 'swap cell state', [p]: 'pause/resume game', [c]: 'clear grid', [arrows]: 'move cursor'";


struct Cursor {
    x: usize,
    y: usize,
}


struct Game {
    running_time: u16,
    paused: bool,
    cols: f64,
    rows: f64,
    show_history: bool,
    cursor: Cursor,
}


impl Game {
    fn new(c: f64, r: f64) -> Game {
        Game {
            running_time: 0,
            paused: true, 
            cols: c,
            rows: r, 
            show_history: false, 
            cursor: Cursor {
                x: (c as usize)/2,
                y: (r as usize)/2,
            }, 
        }
    }
}


struct Population {
    current_generation: Vec<Vec<i8>>,
    dying_generation: Vec<(f64, f64)>,
    ghost_generation: Vec<(f64, f64)>,
    cols: usize,
    rows: usize,
}


impl Population {
    fn new(c: usize, r: usize) -> Population {
        Population {
            current_generation: vec![vec![0; c]; r],
            dying_generation: Vec::new(),
            ghost_generation: Vec::new(),
            cols: c,
            rows: r,
        }
    }

    fn get_next_gen(&mut self) {
        let mut next_gen: Vec<Vec<i8>> = vec![vec![0; self.cols]; self.rows];

        for y in 0..self.rows {
            for x in 0..self.cols {
                let cell_state = self.current_generation[y][x];

                // count neighbors \\
                let mut live_neighbors = -cell_state;
                for i in -1i16..=1 {
                    for j in -1i16..=1 {
                        let new_x = (x as i16) + i;
                        let new_y = (y as i16) + j;

                        if new_x > 0 && new_y > 0 && new_x < self.cols as i16 && new_y < self.rows as i16 {
                            live_neighbors += self.current_generation[new_y as usize][new_x as usize];
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
        self.ghost_generation = self.dying_generation.clone();
        self.dying_generation = Vec::new();
        for y in 0..self.rows {
            for x in 0..self.cols {
                if self.current_generation[y][x] == 1 {
                    self.dying_generation.push((x as f64, y as f64));
                }
            }
        }
        self.current_generation = next_gen;
    }

    fn switch(&mut self, x: usize, y: usize) {
        self.current_generation[y][x] = 1 - self.current_generation[y][x]
    }

    fn kill_all(&mut self) {
        self.current_generation = vec![vec![0; self.cols]; self.rows];
        self.dying_generation = Vec::new();
        self.ghost_generation = Vec::new();
    }
}


pub fn run_gol<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // grid creation \\
    let (c, r) = size().unwrap();
    let (cols, rows) = ((c-2) as usize, (r-5) as usize);
    let mut population = Population::new(cols, rows);
    let mut game = Game::new(cols as f64, rows as f64);

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(50);
    let mut frame_rate: u64 = 400;
    
    loop {
        terminal.draw(|f| ui(f, &game, &population))?;

        // time update \\
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // input handler \\
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => game.paused = !game.paused,
                    KeyCode::Char('h') => game.show_history = !game.show_history,
                    KeyCode::Char('n') => if game.paused {population.get_next_gen()},
                    KeyCode::Char('c') => {
                        population.kill_all();
                        game.paused = true;
                        game.running_time = 0;
                    }
                    KeyCode::Char('s') => {
                        population.switch(game.cursor.x, game.cursor.y);
                    }
                    KeyCode::Enter => population.switch(game.cursor.x, game.cursor.y),
                    KeyCode::Left => if game.cursor.x > 0 {game.cursor.x -= 1},
                    KeyCode::Right => if game.cursor.x < game.cols as usize {game.cursor.x += 1},
                    KeyCode::Up => if game.cursor.y < game.rows as usize {game.cursor.y += 1},
                    KeyCode::Down => if game.cursor.y > 0 {game.cursor.y -= 1},
                    KeyCode::Char('+') => if frame_rate < 1000 {frame_rate += 50},
                    KeyCode::Char('-') => if frame_rate > 50 {frame_rate -= 50},
                    _ => {}
                }
            }
        }

        // grid update \\
        if last_tick.elapsed() >= Duration::from_millis(frame_rate) {
            if !game.paused {
                population.get_next_gen();
                game.running_time += frame_rate as u16;
            }
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, game: &Game, population: &Population) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(f.size());
    
    // controls information \\
    let paragraph = Paragraph::new(HELP)
        .block(Block::default().title("[Help]").borders(Borders::ALL));
    f.render_widget(paragraph, chunks[0]);
    
    // canvas \\
    let pause_span = if game.paused {Span::styled("paused", Style::default().fg(Color::Red))}
    else {Span::styled("playing", Style::default().fg(Color::Green))};
    let title = vec![
        Span::raw("[Game of Life: "),
        pause_span,
        Span::raw(format!("| Timer: {}]", game.running_time/1000))
    ];
    let canvas = Canvas::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .x_bounds([0.0, game.cols])
        .y_bounds([0.0, game.rows])
        .marker(symbols::Marker::Block)
        .paint(|ctx| {
            if game.show_history {
                for (x, y) in population.ghost_generation.clone() {
                    ctx.print(x, y, Span::styled("█", Style::default().fg(Color::Rgb(0, 50, 50))))
                }
                for (x, y) in population.dying_generation.clone() {
                    ctx.print(x, y, Span::styled("█", Style::default().fg(Color::Rgb(0, 100, 100))))
                }
            }
            for y in 0..(game.rows as usize) {
                for x in 0..(game.cols as usize) {
                    if population.current_generation[y][x] == 1 {
                        ctx.print(x as f64, y as f64, Span::styled("█", Style::default().fg(Color::Rgb(0, 255, 255))))
                    }
                }
            }
            ctx.print(game.cursor.x as f64, game.cursor.y as f64, Span::styled("█", Style::default().fg(Color::White)))
        });
    f.render_widget(canvas, chunks[1]);
}
