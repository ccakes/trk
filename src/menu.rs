use termion::{color, cursor};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

use std::io::{self, Write};

pub struct Menu {
    note: String,
    items: Vec<String>,
    selected: usize,
}

#[allow(unused)]
impl Menu {
    pub fn new(note: &str) -> Self {
        Menu {
            note: note.into(),
            items: vec![],
            selected: 0
        }
    }

    pub fn from_vec(note: &str, items: &[&str]) -> Self {
        let items: Vec<_> = items.iter()
            .map(|i| i.to_string())
            .collect();

        Menu {
            note: note.into(),
            items,
            selected: 0
        }
    }

    pub fn add_item<T: Into<String>>(&mut self, item: T) {
        self.items.push(item.into());
    }

    pub fn show(&mut self) -> &str {
        let stdout = io::stdout();
        let stdout = stdout.lock().into_raw_mode().unwrap();

        let mut screen = AlternateScreen::from(stdout);
        write!(screen, "{}", cursor::Hide);

        self.redraw(&mut screen);

        let stdin = io::stdin();
        let stdin = stdin.lock();

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Down => {
                    match self.selected.checked_add(1) {
                        Some(n) if n < self.items.len() => self.selected = n,
                        _ => {}
                    };
                },
                Key::Up => {
                    match self.selected.checked_sub(1) {
                        Some(n) => self.selected = n,
                        None => {}
                    }
                },
                Key::Char('\n') => {
                    self.cleanup(screen);
                    break;

                },
                Key::Char('q') | Key::Esc | Key::Ctrl('c') => {
                    self.cleanup(screen);
                    ::std::process::exit(0);
                }
                _ => {}
            };

            self.redraw(&mut screen);
        }

        &self.items[self.selected]
    }

    fn redraw<W: Write>(&mut self, screen: &mut W) {
        write!(screen, "{}", cursor::Goto(3, 2));
        write!(screen, "{}", self.note);
        
        for (i, item) in self.items.iter().enumerate() {
            let line = 4u16 + i as u16;
            write!(screen, "{}", cursor::Goto(5, line));

            // Highlight selected
            let (colour, sigil) = if self.selected == i {
                (color::AnsiValue::rgb(5, 5, 5), 'o')
            } else {
                (color::AnsiValue::rgb(2, 2, 2), ' ')
            };

            write!(
                screen,
                "{}{} {}{}{}",
                color::Fg(color::LightGreen),
                sigil,
                color::Fg(colour),
                item,
                color::Fg(color::Reset)
            );
        }

        screen.flush().unwrap();
    }

    fn cleanup<W: Write>(&self, mut screen: W) {
        write!(screen, "{}", ToMainScreen);
        write!(screen, "{}", cursor::Show).unwrap();

        drop(screen);

        io::stdout().flush().unwrap();
    }
}
