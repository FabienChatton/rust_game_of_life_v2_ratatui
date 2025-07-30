use std::io;

use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::DefaultTerminal;
use ratatui::Frame;

use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;

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
    game_table_size: (usize, usize)
}
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let terminal_size = terminal.size()?;
        self.game_table_size = (terminal_size.height as usize, terminal_size.width as usize);
        self.game_table = initialize_game_table(self.game_table_size);
        let mut last_update = Instant::now();
        while !self.exit {
            if Instant::now() - last_update >= Duration::from_millis(100) {
                self.game_table = self.update_game_table(self.game_table.clone());
                last_update = Instant::now();
            }
            terminal.draw(|frame| self.draw(frame))?;
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
            _ => {}
        }
    }

    fn count_number_of_neighbour(&self, game_table: &GameTable, x: u16, y: u16) -> u8 {
        let mut count = 0;
        for iy in -1..=1 {
            for ix in -1..=1 {
                if iy == 0 && ix == 0 { continue };
                let real_x = (x as i32 + ix + self.game_table_size.0 as i32) % self.game_table_size.0 as i32;
                let real_y = (y as i32 + iy + self.game_table_size.1 as i32) % self.game_table_size.1 as i32;

                if game_table[real_x as usize][real_y as usize] {
                    count += 1;
                }
            }
        }

        count
    }

    fn update_game_table(&self, game_table: GameTable) -> GameTable {
        let mut new_game_table: GameTable = initialize_empty_game_table(self.game_table_size);
        let game_table_clone = game_table.clone();

        for (x, row) in game_table.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                let neighbour = self.count_number_of_neighbour(&game_table_clone, x as u16, y as u16);
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

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let table = print_game_table(&self.game_table);
        Paragraph::new(Text::from(table))
            .render(area, buf);
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

fn print_game_table(game_table: &GameTable) -> String {
    let mut s_buffer = String::new();
    for row in game_table.iter() {
        for cell in row.iter() {
            if *cell {
                s_buffer.push('#');
            } else {
                s_buffer.push(' ');
            }
        }
        s_buffer.push('\n');
    }
    s_buffer
}