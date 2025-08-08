use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Paragraph};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use std::io;

use rand::Rng;
use ratatui::style::{Color, Style, Stylize};
use std::time::{Duration, Instant};

type GameTable = Vec<Vec<bool>>;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Default)]
struct App {
    exit: bool,
    game_table: GameTable,
    game_table_size: (usize, usize),
    time_to_update: Duration,
    time_to_draw: Duration,
    game_pause: bool,
    game_table_user_cursor: (usize, usize),
}
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let terminal_size = terminal.size()?;
        self.game_table_size = (terminal_size.height as usize, terminal_size.width as usize);
        self.game_table = initialize_game_table(self.game_table_size);
        let mut last_update = Instant::now();
        while !self.exit {
            if !self.game_pause {
                if Instant::now() - last_update >= Duration::from_millis(100) {
                    let time_to_update_t1 = Instant::now();
                    self.game_table = self.update_game_table(self.game_table.clone());
                    self.time_to_update = time_to_update_t1.elapsed();
                    last_update = Instant::now();
                }
            }
            let time_to_draw_t1 = Instant::now();
            terminal.draw(|frame| self.draw(frame))?;
            self.time_to_draw = time_to_draw_t1.elapsed();
            self.handle_events()?
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.toggle_game_pause(),
            KeyCode::Left => self.game_table_user_cursor_move_left(),
            KeyCode::Right => self.game_table_user_cursor_move_right(),
            KeyCode::Up => self.game_table_user_cursor_move_up(),
            KeyCode::Down => self.game_table_user_cursor_move_down(),
            KeyCode::Char('s') => self.switch_cell_state(),
            _ => {}
        }
    }

    fn count_number_of_neighbour(&self, game_table: &GameTable, x: u16, y: u16) -> u8 {
        let xi32 = x as i32;
        let yi32 = y as i32;
        let game_table_size0i32 = self.game_table_size.0 as i32;
        let game_table_size1i32 = self.game_table_size.1 as i32;
        let mut count = 0;
        for iy in -1..=1 {
            for ix in -1..=1 {
                if iy == 0 && ix == 0 { continue };
                let real_x = (xi32 + ix + game_table_size0i32) % game_table_size0i32;
                let real_y = (yi32 + iy + game_table_size1i32) % game_table_size1i32;

                if game_table[real_x as usize][real_y as usize] {
                    count += 1;
                }
            }
        }

        count
    }

    fn update_game_table(&self, game_table: GameTable) -> GameTable {
        let mut new_game_table: GameTable = initialize_empty_game_table(self.game_table_size);

        for (x, row) in game_table.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                let neighbour = self.count_number_of_neighbour(&game_table, x as u16, y as u16);
                let new_cell_state = &mut new_game_table[x][y];
                match (neighbour, *cell) {
                    (2 | 3, true) => *new_cell_state = true,
                    (3, false) => *new_cell_state = true,
                    (_, _) => ()
                }
            }
        }

        new_game_table
    }

    fn toggle_game_pause(&mut self) {
        self.game_pause = !self.game_pause;
    }

    fn game_table_user_cursor_move_left(&mut self) {
        if self.game_table_user_cursor.1 > 0 {
            self.game_table_user_cursor.1 -= 1;
        } else {
            self.game_table_user_cursor.1 = self.game_table_size.1 - 1;
        }
    }

    fn game_table_user_cursor_move_right(&mut self) {
        if self.game_table_user_cursor.1 < self.game_table_size.1 - 1 {
            self.game_table_user_cursor.1 += 1;
        } else {
            self.game_table_user_cursor.1 = 0;
        }
    }

    fn game_table_user_cursor_move_up(&mut self) {
        if self.game_table_user_cursor.0 > 0 {
            self.game_table_user_cursor.0 -= 1;
        } else {
            self.game_table_user_cursor.0 = self.game_table_size.0 - 1;
        }
    }

    fn game_table_user_cursor_move_down(&mut self) {
        if self.game_table_user_cursor.0 < self.game_table_size.0 - 1 {
            self.game_table_user_cursor.0 += 1;
        } else {
            self.game_table_user_cursor.0 = 0;
        }
    }

    fn print_game_table(&self) -> Text {
        let mut lines = Vec::new();

        for (x, row) in self.game_table.iter().enumerate() {
            let mut spans = Vec::new();
            for (y, cell) in row.iter().enumerate() {
                let character = if *cell { "#" } else { " " };
                let span = if self.game_pause &&
                    x == self.game_table_user_cursor.0 && y == self.game_table_user_cursor.1
                {
                    Span::styled(character, Style::default().bg(Color::LightGreen))
                } else {
                    Span::styled(character, Style::default())
                };
                spans.push(span);
            }
            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }

    fn switch_cell_state(&mut self) {
        if self.game_pause {
            let x = self.game_table_user_cursor.0;
            let y = self.game_table_user_cursor.1;

            self.game_table[x][y] = !self.game_table[x][y];

        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let game_table_printed = self.print_game_table();
        let instructions = Line::from(vec![
            "Quit".into(),
            " <q>".bold().blue(),
            ", Time to update [ms]".into(),
            format!(" {}", self.time_to_update.as_millis()).blue(),
            ", Time to draw [ms]".into(),
            format!(" {}", self.time_to_draw.as_millis()).blue(),
            ", Pause".into(),
            " <Space>".bold().blue(),
            " Move cursor while Pause with".into(),
            " <Arrow>".bold().blue(),
            " Switch cell state".into(),
            " <s>".bold().blue(),
        ]);

        Paragraph::new(game_table_printed)
            .block(Block::new().title(instructions.centered()))
            .render(area, buf)
    }
}

fn initialize_game_table(size: (usize, usize)) -> GameTable {
    let mut game_table: GameTable = Vec::new();
    for _ in 0..size.0 {
        let mut row: Vec<bool> = Vec::new();
        for _ in 0..size.1 {
            let rng_bool: bool = rand::rng().random();
            row.push(rng_bool);
        }
        game_table.push(row);
    }

    game_table
}

fn initialize_empty_game_table(size: (usize, usize)) -> GameTable {
    let mut game_table: GameTable = Vec::new();
    for _ in 0..size.0 {
        let mut row: Vec<bool> = Vec::new();
        for _ in 0..size.1 {
            row.push(false);
        }
        game_table.push(row);
    }

    game_table
}