use crossterm::{
    cursor::position,
    event::{self, read, Event, KeyCode, KeyEvent},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::time::Duration;

const RESERVED_WORDS: [&str; 3] = ["hello", "print", "ping"];

pub fn read_line() -> io::Result<String> {
    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Enter => {
                break;
            }
            KeyCode::Char(c) => {
                line.push(c);
            }
            KeyCode::Esc => {
                break;
            }
            _ => {}
        }
    }
    Ok(line)
}

fn main() -> io::Result<()> {
    let line = read_line()?;

    let output = String::from_iter(line.split(" ").map(|ex| {
        if RESERVED_WORDS.contains(&ex) {
            format!("{} ", ex.yellow())
        } else {
            format!("{} ", ex.green())
        }
    }));

    println!("read line:");
    println!("{output}");

    Ok(())
}
