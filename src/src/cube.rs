use std::{io, format, time::{Duration, Instant}};
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph, canvas::{Canvas, Line}},
    layout::{Layout, Constraint},
    style::Color,
    symbols,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::size,
};

const HELP: &str = "[r]: 'reset cube', [arrows]: 'move cube'";


struct Cube {
    theta: f64,
    theta_speed: f64,
    sigma: f64,
    sigma_speed: f64,
    verticies: Vec<(f64, f64, f64)>,
    scheme: Vec<(usize, usize)>,
}

impl Cube {
    fn new() -> Cube {
        let s: f64 = 30.0;
        Cube {
            theta: 0.0,
            theta_speed: 0.0,
            sigma: 0.0,
            sigma_speed: 0.0,
            verticies: vec![
                (-s, -s, -s),
                (s, -s, -s),
                (s, s, -s),
                (-s, s, -s),
                (-s, -s, s),
                (s, -s, s),
                (s, s, s),
                (-s, s, s),
            ],
            scheme: vec![
                (0, 1), (1, 2), (2, 3), (3, 0),
                (4, 5), (5, 6), (6, 7), (7, 4),
                (0, 4), (1, 5), (2, 6), (3, 7),
            ],
        }
    }

    fn rotation(&self, origin_x: f64, origin_y: f64) -> Vec<Line> {
        // angle conversion \\
        let theta = self.theta.to_radians();
        let sigma = self.sigma.to_radians();

        // rotation calculation \\
        let mut temp_verticies: Vec<(f64, f64, f64)> = Vec::new();
        for (x, y, z) in &self.verticies {
            let (x, y, z) = (x, y*theta.cos() - z*theta.sin(), y*theta.sin() + z*theta.cos());
            let (x, y, z) = (x*sigma.cos() + z*sigma.sin(), y, -x*sigma.sin() + z*sigma.cos());
            temp_verticies.push((x, y, z));
        }

        // converting verticies coordinates to Line struct for drawing \\
        let mut lines: Vec<Line> = Vec::new();
        for (p1, p2) in &self.scheme {
            let (x1, y1, _z1) = temp_verticies[*p1];
            let (x2, y2, _z2) = temp_verticies[*p2];
            let line = Line {
                x1: x1 + origin_x,
                x2: x2 + origin_x,
                y1: y1 + origin_y,
                y2: y2 + origin_y,
                color: Color::Cyan,
            };
            lines.push(line);
        }
        lines
    }

    fn reset(&mut self) {
        self.theta = 0.0;
        self.theta_speed = 0.0;
        self.sigma = 0.0;
        self.sigma_speed = 0.0;
    }
}


pub fn run_cube<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // cube creation \\
    let (c, r) = size().unwrap();
    let (cols, rows) = (((c-2)*2) as f64, ((r-5)*3) as f64);
    let origin_x: f64 = cols / 2.0;
    let origin_y: f64 = rows / 2.0;
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(50);
    let mut cube = Cube::new();

    loop {
        cube.theta += cube.theta_speed;
        cube.sigma += cube.sigma_speed;
        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(2)])
                .split(f.size());
            
            // controls information \\
            let paragraph = Paragraph::new(HELP)
                .block(Block::default().title("[Help]").borders(Borders::ALL));
            f.render_widget(paragraph, chunks[0]);
            
            // canvas \\
            let title = format!("[Cube: sigma={}, theta={}]", cube.sigma, cube.theta);
            let canvas = Canvas::default()
                .block(Block::default().title(title).borders(Borders::ALL))
                .x_bounds([0.0, (cols-1.0) as f64])
                .y_bounds([0.0, (rows-1.0) as f64])
                .marker(symbols::Marker::Braille)
                .paint(|ctx| {
                    for line in cube.rotation(origin_x, origin_y) {ctx.draw(&line)}
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
                    KeyCode::Char('r') => cube.reset(),
                    KeyCode::Left => cube.sigma_speed += 0.25,
                    KeyCode::Right => cube.sigma_speed -= 0.25,
                    KeyCode::Up => cube.theta_speed += 0.25,
                    KeyCode::Down => cube.theta_speed -= 0.25,
                    _ => {}
                }
            }
        }

        // grid update \\
        if last_tick.elapsed() >= Duration::from_millis(400) {
            last_tick = Instant::now();
        }
    }
}
