use std::{io, format, time::{Duration, Instant}};
use tui::{
    backend::Backend,
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

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
    Idle,
}


#[derive(PartialEq)]
struct Snake {
    body: Vec<(f64, f64)>,
    direction: Direction,
    dead: bool,
    color: Color,
}

impl Snake {
    fn new(x: f64, y: f64) -> Snake {
        Snake {
            body: vec![(x, y)],
            direction: Direction::Idle,
            dead: false,
            color: Color::LightCyan,
        }
    }

    fn update(&mut self, cols: u32, rows: u32) {
        let (mut x, mut y) = self.body.last().unwrap();
        match self.direction {
            Direction::Left => x -= 1.0,
            Direction::Right => x += 1.0,
            Direction::Up => y += 1.0,
            Direction::Down => y -= 1.0,
            _ => {}
        };
        if self.direction != Direction::Idle {
            if self.body.contains(&(x, y)) || !(0.0<=x&&x<cols as f64) || !(0.0<=y&&y<rows as f64) {
                self.dead = true;
                self.direction = Direction::Idle;
                self.color = Color::Red;
            }
            self.body.remove(0);
            self.body.push((x, y));
        }
    }

    fn go(&mut self, direction: Direction) {
        self.direction = direction;
    }

}



fn summon_apple(snake: &Snake, cols: u32, rows: u32) -> (f64, f64) {
    let mut x = rand::thread_rng().gen_range(0..cols);
    let mut y = rand::thread_rng().gen_range(0..rows);
    while snake.body.contains(&(x as f64, y as f64)) {
        x = rand::thread_rng().gen_range(0..cols);
        y = rand::thread_rng().gen_range(0..rows);
    }
    (x as f64, y as f64)
}




fn snake_eats_apple(snake: &Snake, apple_coords: (f64, f64)) -> bool {
    snake.body.contains(&apple_coords)
}


pub fn run_snake<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let (c, r) = size().unwrap();
    let (cols, rows) = ((c-2) as u32, (r-5) as u32);

    // snake creation \\
    let mut snake: Snake = Snake::new((cols/2) as f64, (rows/2) as f64);
    
    let mut apple_coords = summon_apple(&snake, cols, rows);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

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
            let title = format!("[Snake: size={}]", snake.body.len());
            let canvas = Canvas::default()
                .block(Block::default().title(title).borders(Borders::ALL))
                .x_bounds([0.0, (cols-1) as f64])
                .y_bounds([0.0, (rows-1) as f64])
                .marker(symbols::Marker::Block)
                .paint(|ctx| {
                    for (x, y) in snake.body.clone() {
                        ctx.print(x, y, Span::styled("█", Style::default().fg(snake.color)))
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
                        snake = Snake::new((cols/2) as f64, (rows/2) as f64);
                        apple_coords = summon_apple(&snake, cols, rows);
                    },
                    KeyCode::Left => snake.go(Direction::Left),
                    KeyCode::Right => snake.go(Direction::Right),
                    KeyCode::Up => snake.go(Direction::Up),
                    KeyCode::Down => snake.go(Direction::Down),
                    _ => {}
                }
            }
        }
        // game update \\
        if last_tick.elapsed() >= tick_rate {
            if !snake.dead {
                snake.update(cols, rows);
                if snake_eats_apple(&snake, apple_coords) {
                    snake.body.push(apple_coords);
                    apple_coords = summon_apple(&snake, cols, rows);
                }
            } else if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => {
                        snake = Snake::new((cols/2) as f64, (rows/2) as f64);
                        apple_coords = summon_apple(&snake, cols, rows);
                    },
                    _ => {}
                }
            }
            last_tick = Instant::now();
        }
    }
}
