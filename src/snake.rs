use std::{io, format, time::{Duration, Instant}};
use tui::{
    backend::{Backend},
    widgets::{Block, Borders, Paragraph, canvas::{Canvas, Points}},
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
use rand::Rng;

const HELP: &str = "[r]: 'reset game', [arrows]: 'change direction'";


fn summon_apple(snake: &mut Vec<(f64, f64)>, cols: u32, rows: u32) -> (f64, f64) {
    let mut x: f64 = rand::thread_rng().gen_range(0..cols) as f64;
    let mut y: f64 = rand::thread_rng().gen_range(0..rows) as f64;
    while snake.contains(&(x, y)) {
        x = rand::thread_rng().gen_range(0..101) as f64;
        y = rand::thread_rng().gen_range(0..101) as f64;
    }
    (x as f64, y as f64)
}


fn update_snake(snake: &mut Vec<(f64, f64)>, direction: &str, cols: u32, rows: u32) -> bool {
    let (mut x, mut y) = snake.last().unwrap();
    match direction {
        "left" => (x, y) = (x-1.0, y),
        "right" => (x, y) = (x+1.0, y),
        "up" => (x, y) = (x, y+1.0),
        "down" => (x, y) = (x, y-1.0),
        _ => {}
    };
    if direction != "idle" {
        if snake.contains(&(x, y)) || !(0.0<=x&&x<cols as f64) || !(0.0<=y&&y<rows as f64) {
            return true;
        }
        snake.remove(0);
        snake.push((x, y));
    }
    false
}


fn snake_eats_apple(snake: &mut Vec<(f64, f64)>, apple_coords: (f64, f64)) -> bool {
    snake.contains(&apple_coords)
}


pub fn run_snake<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let (c, r) = size().unwrap();
    let (cols, rows) = ((c-2) as u32, (r-5) as u32);
    // snake creation \\
    let mut snake = Vec::new();
    snake.push(((cols/2) as f64, (rows/2) as f64));
    let mut snake_color = Color::Green;
    let mut direction = "idle";
    
    let mut apple_coords = summon_apple(&mut snake, cols, rows);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);
    let mut game_over = false;

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
            let title = format!("[Snake: size={}]", snake.len());
            let canvas = Canvas::default()
                .block(Block::default().title(title).borders(Borders::ALL))
                .x_bounds([0.0, (cols-1) as f64])
                .y_bounds([0.0, (rows-1) as f64])
                .marker(symbols::Marker::Block)
                .paint(|ctx| {
                    for (x, y) in snake.clone() {
                        ctx.print(x, y, Span::styled("█", Style::default().fg(snake_color)))
                    }
                    ctx.draw(&Points {
                        coords: &[apple_coords],
                        color: Color::Red
                    });
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
                    KeyCode::Char('r') => {
                        snake = Vec::new();
                        snake.push(((cols/2) as f64, (rows/2) as f64));
                        snake_color = Color::Green;
                        direction = "idle";
                        apple_coords = summon_apple(&mut snake, cols, rows);
                        game_over = false
                    },
                    KeyCode::Left => if direction != "right" {direction = "left"},
                    KeyCode::Right => if direction != "left" {direction = "right"},
                    KeyCode::Up => if direction != "down" {direction = "up"},
                    KeyCode::Down => if direction != "up" {direction = "down"},
                    _ => {}
                }
            }
        }
        // grid update \\
        if last_tick.elapsed() >= tick_rate {
            if !game_over {
                game_over = update_snake(&mut snake, direction, cols, rows);
                if snake_eats_apple(&mut snake, apple_coords) {
                    snake.push(apple_coords);
                    apple_coords = summon_apple(&mut snake, cols, rows);
                }
            } else {
                snake_color = Color::Red;
                direction = "idle"
            }
            last_tick = Instant::now();
        }
    }
}
