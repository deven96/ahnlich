use crossterm::event::{
    poll, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, SetForegroundColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use std::io::{self, stdout, Stdout, Write};

use crate::connect::AgentPool;

const RESERVED_WORDS: [&str; 3] = ["ping", "infoserver", "createpredindex"];

#[derive(Debug)]
enum SpecialEntry {
    Enter,
    Up,
    Down,
    Break,
    Left,
    Right,
}

#[derive(Debug)]
enum Entry {
    Char(char),
    Special(SpecialEntry),
    Other(KeyCode),
    None,
}

pub struct Term {
    client_pool: AgentPool,
}

impl Term {
    pub fn new(client_pool: AgentPool) -> Self {
        Self { client_pool }
    }

    pub(crate) fn read_char(&self) -> io::Result<Entry> {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            Ok(match code {
                KeyCode::Enter => Entry::Special(SpecialEntry::Enter),
                KeyCode::Char(c) => Entry::Char(c),
                KeyCode::Left => Entry::Special(SpecialEntry::Left),
                KeyCode::Up => Entry::Special(SpecialEntry::Up),
                KeyCode::Down => Entry::Special(SpecialEntry::Down),
                KeyCode::Right => Entry::Special(SpecialEntry::Right),
                _ => Entry::Other(code),
            })
        } else {
            Ok(Entry::None)
        }
    }
    pub fn welcome_message(&self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(format!(
            "Welcome To Ahnlich {}\n\n",
            self.client_pool
        )))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn ahnlich_prompt(&self, stdout: &mut Stdout) -> io::Result<()> {
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(">>> "))?;
        stdout.execute(SetForegroundColor(Color::White))?;

        stdout.flush()?;
        Ok(())
    }

    //  pub(crate) fn print_query(&self, query: &str) -> io::Result<()> {
    //      self.ahnlich_prompt()?;
    //      let output = String::from_iter(query.split(' ').map(|ex| {
    //          if RESERVED_WORDS.contains(&(ex.to_lowercase().as_str())) {
    //              format!("{} ", ex.magenta())
    //          } else {
    //              format!("{} ", ex.white())
    //          }
    //      }));

    //      println!("{output}");

    //      Ok(())
    //  }

    fn read_line(&self, stdout: &mut Stdout) -> io::Result<String> {
        let (start_pos_col, _) = cursor::position()?;
        let mut output = String::new();

        loop {
            let char = self.read_char()?;
            let (current_pos_col, _) = cursor::position()?;
            match char {
                Entry::Char(c) => {
                    output.push(c);
                    stdout.execute(Print(c))?;
                    stdout.flush()?;
                }
                Entry::Special(special) => match special {
                    SpecialEntry::Up | SpecialEntry::Down => {
                        continue;
                    }
                    SpecialEntry::Enter | SpecialEntry::Break => {
                        queue!(stdout, Print("\n"), cursor::MoveToColumn(0))?;
                        stdout.flush()?;
                        break;
                    }
                    SpecialEntry::Left => {
                        if start_pos_col < current_pos_col {
                            stdout.execute(cursor::MoveLeft(1))?;
                        }
                    }
                    SpecialEntry::Right => {
                        if start_pos_col + output.len() as u16 > current_pos_col {
                            stdout.execute(cursor::MoveRight(1))?;
                        }
                    }
                },
                Entry::Other(_) | Entry::None => {
                    continue;
                }
            }
        }
        Ok(output)
    }

    pub async fn run(&self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(cursor::EnableBlinking)?;
        stdout.execute(cursor::SetCursorStyle::BlinkingBar)?;

        loop {
            self.ahnlich_prompt(&mut stdout)?;
            let input = self.read_line(&mut stdout)?;
            match input.as_str() {
                "quit" | "exit()" => break,
                command => {
                    let response = self.client_pool.parse_queries(command).await;

                    match response {
                        Ok(success) => {
                            for msg in success {
                                queue!(
                                    stdout,
                                    Print(format!("{}\n", msg)),
                                    cursor::MoveToColumn(0)
                                )?;
                            }
                            stdout.flush()?;
                        }
                        Err(err) => {
                            queue!(
                                stdout,
                                Print(format!("{}\n", err.red())),
                                cursor::MoveToColumn(0)
                            )?;
                            stdout.flush()?;
                        }
                    }
                }
            };
        }
        disable_raw_mode()?;
        Ok(())
    }
}
